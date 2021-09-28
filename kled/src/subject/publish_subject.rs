use crate::cancellable::*;
use crate::core;
use crate::observable;
use async_std::sync::Mutex;
use async_trait::async_trait;
#[chronobreak]
use std::sync::Arc;

pub struct PublishSubject<Cancellable, Item, Error> {
    data: Arc<Mutex<Data<Cancellable, Item, Error>>>,
}

struct Data<Cancellable, Item, Error> {
    cancellable: Option<Cancellable>,
    emitters: Vec<observable::BoxEmitter<Item, Error>>,
}

impl<Cancellable, Item, Error> Default for PublishSubject<Cancellable, Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                cancellable: None,
                emitters: vec![],
            })),
        }
    }
}

impl<Cancellable, Item, Error> Clone for PublishSubject<Cancellable, Item, Error> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

#[async_trait]
impl<Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for PublishSubject<Cancellable, Item, Error>
where
    Cancellable: Send,
    Item: Clone + Send,
    Error: Clone + Send,
{
    async fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.data.lock().await.cancellable = Some(cancellable);
    }

    async fn on_next(&mut self, item: Item) {
        let mut data = self.data.lock().await;
        let count = data.emitters.len();
        match count {
            0 => (),
            1 => data.emitters[0].on_next(item).await,
            len => {
                for e in data.emitters.iter_mut().take(len - 1) {
                    let item = item.clone();
                    e.on_next(item).await
                }
                data.emitters[len - 1].on_next(item).await;
            }
        }
    }

    async fn on_error(&mut self, error: Error) {
        let mut data = self.data.lock().await;
        let count = data.emitters.len();
        match count {
            0 => (),
            1 => data.emitters[0].on_error(error).await,
            len => {
                for e in data.emitters.iter_mut().take(len - 1) {
                    let error = error.clone();
                    e.on_error(error).await
                }
                data.emitters[len - 1].on_error(error).await;
            }
        }
    }

    async fn on_completed(&mut self) {
        for o in self.data.lock().await.emitters.iter_mut() {
            o.on_completed().await;
        }
    }
}

impl<Cancellable, Item, Error> core::Subject<Cancellable, ArcCancellable, Item, Error>
    for PublishSubject<Cancellable, Item, Error>
where
    Cancellable: Send,
    Item: Clone + Send + 'static,
    Error: Clone + Send + 'static,
{
}

#[async_trait]
impl<Cancellable, Item, Error> core::Observable<ArcCancellable, Item, Error>
    for PublishSubject<Cancellable, Item, Error>
where
    Cancellable: Send,
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<ArcCancellable, Item, Error> + Send + 'static,
    {
        let mut lock = self.data.lock().await;
        lock.emitters
            .push(observable::BoxEmitter::from(observer).await)
    }
}

#[cfg(test)]
mod tests {
    use super::PublishSubject;
    use crate::observable::*;
    use crate::observer::*;
    use crate::prelude::*;
    use async_std::sync::Barrier;
    use async_std::task;
    #[chronobreak]
    use std::sync::Arc;

    #[async_std::test]
    async fn simple() {
        let subject = PublishSubject::default();

        let subject2 = subject.clone();
        let barrier = Arc::new(Barrier::new(2));
        let barrier2 = barrier.clone();
        let handle = task::spawn(async move {
            barrier2.wait().await;
            vec![0, 1, 2, 3].into_observable().subscribe(subject2).await
        });

        let test_observer1 = TestObserver::default();
        subject.clone().subscribe(test_observer1.clone()).await;

        barrier.wait().await;
        handle.await;

        let test_observer2 = TestObserver::default();
        subject.subscribe(test_observer2.clone()).await;

        assert_eq!(test_observer1.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer1.items().await, vec![0, 1, 2, 3]);
        assert_eq!(test_observer2.status().await, ObserverStatus::Subscribed);
        assert_eq!(test_observer2.items().await, vec![]);
    }

    #[async_std::test]
    async fn interleaved() {
        let subject = PublishSubject::default();
        let test_observable = TestObservable::default().annotate_error_type(());
        test_observable.clone().subscribe(subject.clone()).await;

        let test_observer1 = TestObserver::default();
        subject.clone().subscribe(test_observer1.clone()).await;

        test_observable.emit(0).await;

        let test_observer2 = TestObserver::default();
        subject.subscribe(test_observer2.clone()).await;

        test_observable.emit_all(vec![1, 2, 3]).await;
        test_observable.emit_on_completed().await;

        assert_eq!(test_observer1.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer1.items().await, vec![0, 1, 2, 3]);
        assert_eq!(test_observer2.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer2.items().await, vec![1, 2, 3]);
    }

    #[async_std::test]
    async fn error() {
        let subject = PublishSubject::default();
        let test_observable = TestObservable::default().annotate_item_type(());
        test_observable.clone().subscribe(subject.clone()).await;

        let test_observer1 = TestObserver::default();
        subject.clone().subscribe(test_observer1.clone()).await;

        test_observable.emit_error(0).await;

        let test_observer2 = TestObserver::default();
        subject.subscribe(test_observer2.clone()).await;

        assert_eq!(test_observer1.status().await, ObserverStatus::Error);
        assert_eq!(test_observer1.error().await, Some(0));
        assert_eq!(test_observer2.status().await, ObserverStatus::Subscribed);
        assert_eq!(test_observer2.error().await, None);
    }
}
