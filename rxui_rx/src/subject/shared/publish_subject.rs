use crate::consumer;
use crate::core;
use crate::core::Consumer;
use std::sync::{Arc, Mutex};

pub struct PublishSubject<Subscription, Item, Error> {
    data: Arc<Mutex<Data<Subscription, Item, Error>>>,
}

struct Data<Subscription, Item, Error> {
    subscription: Option<Subscription>,
    observers: Vec<Box<dyn core::CancellableConsumer<Item, Error> + Send + Sync + 'static>>,
}

impl<Subscription, Item, Error> Default for PublishSubject<Subscription, Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                subscription: None,
                observers: vec![],
            })),
        }
    }
}

impl<Subscription, Item, Error> Clone for PublishSubject<Subscription, Item, Error> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<Subscription, Item, Error> core::Observer<Subscription, Item, Error>
    for PublishSubject<Subscription, Item, Error>
where
    Item: Copy,
    Error: Copy,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        self.data.lock().unwrap().subscription = Some(subscription);
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

impl<Subscription, Item, Error> core::SharedSubject<Subscription, Item, Error>
    for PublishSubject<Subscription, Item, Error>
where
    Item: Copy + Send + Sync + 'static,
    Error: Copy + Send + Sync + 'static,
{
}

impl<Subscription, Item, Error> core::SharedObservable for PublishSubject<Subscription, Item, Error>
where
    Item: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Subscription = core::SharedSubscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::Observer<Self::Subscription, Self::Item, Self::Error> + Send + Sync + 'static,
    {
        self.data
            .lock()
            .unwrap()
            .observers
            .push(Box::new(consumer::shared::AutoOnSubscribe::new(observer)))
    }
}

impl<Subscription, Item, Error> core::Observable for PublishSubject<Subscription, Item, Error> {
    type Item = Item;
    type Error = Error;
}

#[cfg(test)]
mod tests {
    use super::PublishSubject;
    use crate::prelude::*;
    use crate::util::shared::*;
    use std::thread;

    #[test]
    fn publish_subject_simple() {
        let subject = PublishSubject::default().into_shared();

        let subject2 = subject.clone();
        let handle = thread::spawn(move || {
            vec![0, 1, 2, 3]
                .into_shared_observable()
                .subscribe(subject2)
        });

        let test_observer1 = TestObserver::default();
        subject.clone().subscribe(test_observer1.clone());

        handle.join().unwrap();

        let test_observer2 = TestObserver::default();
        subject.subscribe(test_observer2.clone());

        assert_eq!(test_observer1.status(), ObserverStatus::Completed);
        assert_eq!(test_observer1.items(), vec![0, 1, 2, 3]);
        assert_eq!(test_observer2.status(), ObserverStatus::Subscribed);
        assert_eq!(test_observer2.items(), vec![]);
    }

    #[test]
    fn publish_subject_interleaved() {
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
}
