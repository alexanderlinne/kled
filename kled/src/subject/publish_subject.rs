use crate::cancellable::*;
use crate::core;
use crate::observable;
use crate::util::distribute_value;
#[chronobreak]
use parking_lot::Mutex;
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

impl<Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for PublishSubject<Cancellable, Item, Error>
where
    Item: Clone,
    Error: Clone,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.data.lock().cancellable = Some(cancellable);
    }

    fn on_next(&mut self, item: Item) {
        distribute_value(&mut self.data.lock().emitters, |o, i| o.on_next(i), item);
    }

    fn on_error(&mut self, error: Error) {
        distribute_value(&mut self.data.lock().emitters, |o, e| o.on_error(e), error);
    }

    fn on_completed(&mut self) {
        self.data
            .lock()
            .emitters
            .iter_mut()
            .for_each(|o| o.on_completed());
    }
}

impl<Cancellable, Item, Error> core::Subject<Cancellable, BoolCancellable, Item, Error>
    for PublishSubject<Cancellable, Item, Error>
where
    Item: Clone + Send + 'static,
    Error: Clone + Send + 'static,
{
}

impl<Cancellable, Item, Error> core::Observable<BoolCancellable, Item, Error>
    for PublishSubject<Cancellable, Item, Error>
where
    Item: Send + 'static,
    Error: Send + 'static,
{
    fn subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<BoolCancellable, Item, Error> + Send + 'static,
    {
        self.data
            .lock()
            .emitters
            .push(observable::BoxEmitter::from(observer))
    }
}

#[cfg(test)]
mod tests {
    use super::PublishSubject;
    use crate::observable::*;
    use crate::observer::*;
    use crate::prelude::*;
    #[chronobreak]
    use std::sync::{Arc, Barrier};
    #[chronobreak]
    use std::thread;

    #[test]
    fn simple() {
        let subject = PublishSubject::default();

        let subject2 = subject.clone();
        let barrier = Arc::new(Barrier::new(2));
        let barrier2 = barrier.clone();
        let handle = thread::spawn(move || {
            barrier2.wait();
            vec![0, 1, 2, 3].into_observable().subscribe(subject2)
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
        let subject = PublishSubject::default();
        let test_observable = TestObservable::default().annotate_error_type(());
        test_observable.clone().subscribe(subject.clone());

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
        let subject = PublishSubject::default();
        let test_observable = TestObservable::default().annotate_item_type(());
        test_observable.clone().subscribe(subject.clone());

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
