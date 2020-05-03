use crate::consumer;
use crate::core;
use crate::core::Consumer;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PublishSubject<'o, Cancellable, Item, Error> {
    data: Rc<RefCell<Data<'o, Cancellable, Item, Error>>>,
}

struct Data<'o, Cancellable, Item, Error> {
    cancellable: Option<Cancellable>,
    observers: Vec<Box<dyn core::CancellableConsumer<Item, Error> + 'o>>,
}

impl<'o, Cancellable, Item, Error> Default for PublishSubject<'o, Cancellable, Item, Error> {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new(Data {
                cancellable: None,
                observers: vec![],
            })),
        }
    }
}

impl<'o, Cancellable, Item, Error> Clone for PublishSubject<'o, Cancellable, Item, Error> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<'o, Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for PublishSubject<'o, Cancellable, Item, Error>
where
    Item: Copy,
    Error: Copy,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.data.borrow_mut().cancellable = Some(cancellable);
    }

    fn on_next(&mut self, item: Item) {
        &mut self.data.borrow_mut().observers.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        &mut self.data.borrow_mut().observers.on_error(error);
    }

    fn on_completed(&mut self) {
        &mut self.data.borrow_mut().observers.on_completed();
    }
}

impl<'o, Cancellable, Item, Error> core::LocalSubject<'o, Cancellable, Item, Error>
    for PublishSubject<'o, Cancellable, Item, Error>
where
    Item: Copy + 'o,
    Error: Copy + 'o,
{
}

impl<'o, Cancellable, Item, Error> core::LocalObservable<'o>
    for PublishSubject<'o, Cancellable, Item, Error>
where
    Item: 'o,
    Error: 'o,
{
    type Cancellable = core::LocalCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + 'o,
    {
        self.data
            .borrow_mut()
            .observers
            .push(Box::new(consumer::local::AutoOnSubscribe::new(observer)))
    }
}

impl<'o, Cancellable, Item, Error> core::Observable
    for PublishSubject<'o, Cancellable, Item, Error>
{
    type Item = Item;
    type Error = Error;
}

#[cfg(test)]
mod tests {
    use super::PublishSubject;
    use crate::prelude::*;
    use crate::util::local::*;

    #[test]
    fn simple() {
        let subject = PublishSubject::default();

        let test_observer1 = TestObserver::default();
        subject.clone().subscribe(test_observer1.clone());

        vec![0, 1, 2, 3]
            .into_observable()
            .subscribe(subject.clone());

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

    #[test]
    fn no_subscriber() {
        let subject = PublishSubject::default();
        let test_observable = TestObservable::default().annotate_error_type(());
        test_observable.clone().subscribe(subject.clone());
        test_observable.emit(0);
    }

    #[test]
    fn single_subscriber() {
        let subject = PublishSubject::default();
        let test_observable = TestObservable::default().annotate_error_type(());
        test_observable.clone().subscribe(subject.clone());
        test_observable.emit(0);
        let test_observer = TestObserver::default();
        subject.subscribe(test_observer.clone());
    }
}
