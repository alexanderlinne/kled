use crate::cancellable::*;
use crate::core;
use crate::observable;
use async_trait::async_trait;
use std::future::Future;
use std::marker::PhantomData;

#[derive(Clone)]
#[doc(hidden)]
pub struct ObservableCreate<F, Item, Error> {
    emitter_consumer: F,
    phantom: PhantomData<(Item, Error)>,
}

impl<F, Item, Error> ObservableCreate<F, Item, Error> {
    pub fn new(emitter_consumer: F) -> Self {
        ObservableCreate {
            emitter_consumer,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Fn, F, Item, Error> core::Observable<ArcCancellable, Item, Error>
    for ObservableCreate<Fn, Item, Error>
where
    Fn: FnOnce(observable::BoxEmitter<Item, Error>) -> F + Send,
    F: Future + Send,
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<ArcCancellable, Item, Error> + Send + 'static,
    {
        let emitter = observable::BoxEmitter::from(observer).await;
        (self.emitter_consumer)(emitter).await;
    }
}

pub fn create<Fn, F, Item, Error>(emitter_consumer: Fn) -> ObservableCreate<Fn, Item, Error>
where
    Fn: FnOnce(observable::BoxEmitter<Item, Error>) -> F + Send,
    F: Future + Send,
    Item: Send + 'static,
    Error: Send + 'static,
{
    ObservableCreate::new(emitter_consumer)
}

#[cfg(test)]
mod tests {
    use crate::observer::*;
    use crate::prelude::*;

    #[async_std::test]
    async fn create() {
        let test_observer = TestObserver::default();
        observable::create(|mut emitter| async move {
            if false {
                emitter.on_error(()).await
            };
            emitter.on_next(0).await;
            emitter.on_completed().await;
        })
        .subscribe(test_observer.clone())
        .await;
        assert_eq!(test_observer.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer.items().await, vec![0]);
    }
}
