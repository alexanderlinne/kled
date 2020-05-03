use crate::consumer;
use crate::core;
use crate::core::Consumer;
use std::sync::{Arc, Mutex};

pub struct PublishSubject<Cancellable, Item, Error> {
    data: Arc<Mutex<Data<Cancellable, Item, Error>>>,
}

struct Data<Cancellable, Item, Error> {
    cancellable: Option<Cancellable>,
    observers: Vec<Box<dyn core::CancellableConsumer<Item, Error> + Send + 'static>>,
}

impl<Cancellable, Item, Error> Default for PublishSubject<Cancellable, Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                cancellable: None,
                observers: vec![],
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

impl<Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for PublishSubject<Cancellable, Item, Error>
where
    Item: Clone,
    Error: Clone,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.data.lock().unwrap().cancellable = Some(cancellable);
    }

    fn on_next(&mut self, item: Item) {
        &mut self.data.lock().unwrap().observers.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        &mut self.data.lock().unwrap().observers.on_error(error);
    }

    fn on_completed(&mut self) {
        &mut self.data.lock().unwrap().observers.on_completed();
    }
}

impl<Cancellable, Item, Error> core::SharedSubject<Cancellable, Item, Error>
    for PublishSubject<Cancellable, Item, Error>
where
    Item: Clone + Send + 'static,
    Error: Clone + Send + 'static,
{
}

impl<Cancellable, Item, Error> core::SharedObservable for PublishSubject<Cancellable, Item, Error>
where
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Cancellable = core::SharedCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        self.data
            .lock()
            .unwrap()
            .observers
            .push(Box::new(consumer::shared::AutoOnSubscribe::new(observer)))
    }
}

impl<Cancellable, Item, Error> core::Observable for PublishSubject<Cancellable, Item, Error> {
    type Item = Item;
    type Error = Error;
}

#[cfg(test)]
mod tests {
    use super::PublishSubject;
    use crate::prelude::*;
    use crate::util::shared::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn simple() {
        let subject = PublishSubject::default().into_shared();

        let subject2 = subject.clone();
        let barrier = Arc::new(Barrier::new(1));
        let barrier2 = barrier.clone();
        let handle = thread::spawn(move || {
            barrier2.wait();
            vec![0, 1, 2, 3]
                .into_shared_observable()
                .subscribe(subject2)
        });

        let test_observer1 = TestObserver::default();
        subject.clone().subscribe(test_observer1.clone());

        barrier.wait();
        handle.join().unwrap();

        let test_observer2 = TestObserver::default();
        subject.subscribe(test_observer2.clone());

        assert_eq!(test_observer1.status(), ObserverStatus::Completed);
        assert_eq!(test_observer1.items(), vec![0, 1, 2, 3]);
        assert_eq!(test_observer2.status(), ObserverStatus::Subscribed);
        assert_eq!(test_observer2.items(), vec![]);
    }

    #[test]
    fn interleaved() {
        let subject = PublishSubject::default().into_shared();
        let test_observable = TestObservable::default().annotate_error_type(());
        test_observable
            .clone()
            .into_shared()
            .subscribe(subject.clone());

        let test_observer1 = TestObserver::default();
        subject.clone().subscribe(test_observer1.clone());

        test_observable.emit(0);

        let test_observer2 = TestObserver::default();
        subject.subscribe(test_observer2.clone());

        test_observable.emit_all(vec![1, 2, 3]);
        test_observable.emit_on_completed();

        assert_eq!(test_observer1.status(), ObserverStatus::Completed);
        assert_eq!(test_observer1.items(), vec![0, 1, 2, 3]);
        assert_eq!(test_observer2.status(), ObserverStatus::Completed);
        assert_eq!(test_observer2.items(), vec![1, 2, 3]);
    }

    #[test]
    fn error() {
        let subject = PublishSubject::default().into_shared();
        let test_observable = TestObservable::default().annotate_item_type(());
        test_observable
            .clone()
            .into_shared()
            .subscribe(subject.clone());

        let test_observer1 = TestObserver::default();
        subject.clone().subscribe(test_observer1.clone());

        test_observable.emit_error(0);

        let test_observer2 = TestObserver::default();
        subject.subscribe(test_observer2.clone());

        assert_eq!(test_observer1.status(), ObserverStatus::Error);
        assert_eq!(test_observer1.error(), Some(0));
        assert_eq!(test_observer2.status(), ObserverStatus::Subscribed);
        assert_eq!(test_observer2.error(), None);
    }
}
