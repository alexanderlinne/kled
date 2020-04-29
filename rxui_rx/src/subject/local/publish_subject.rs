use crate::consumer;
use crate::core;
use crate::core::Consumer;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PublishSubject<'o, Subscription, Item, Error> {
    data: Rc<RefCell<Data<'o, Subscription, Item, Error>>>,
}

struct Data<'o, Subscription, Item, Error> {
    subscription: Option<Subscription>,
    observers: Vec<Box<dyn core::CancellableConsumer<Item, Error> + 'o>>,
}

impl<'o, Subscription, Item, Error> Default for PublishSubject<'o, Subscription, Item, Error> {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new(Data {
                subscription: None,
                observers: vec![],
            })),
        }
    }
}

impl<'o, Subscription, Item, Error> Clone for PublishSubject<'o, Subscription, Item, Error> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<'o, Subscription, Item, Error> core::Observer<Subscription, Item, Error>
    for PublishSubject<'o, Subscription, Item, Error>
where
    Item: Copy,
    Error: Copy,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        self.data.borrow_mut().subscription = Some(subscription);
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

impl<'o, Subscription, Item, Error> core::LocalSubject<'o, Subscription, Item, Error>
    for PublishSubject<'o, Subscription, Item, Error>
where
    Item: Copy + 'o,
    Error: Copy + 'o,
{
}

impl<'o, Subscription, Item, Error> core::LocalObservable<'o>
    for PublishSubject<'o, Subscription, Item, Error>
where
    Item: 'o,
    Error: 'o,
{
    type Subscription = core::LocalSubscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Subscription, Self::Item, Self::Error> + 'o,
    {
        self.data
            .borrow_mut()
            .observers
            .push(Box::new(consumer::local::AutoOnSubscribe::new(observer)))
    }
}

impl<'o, Subscription, Item, Error> core::Observable
    for PublishSubject<'o, Subscription, Item, Error>
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
    fn publish_subject_simple() {
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
    fn publish_subject_interleaved() {
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
}
