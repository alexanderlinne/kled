use crate::core;
use crate::core::Observer;
use crate::observer;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct FnObservable<F, Item, Error> {
    subscriber_consumer: F,
    phantom: PhantomData<(Item, Error)>,
}

impl<F, Item, Error> FnObservable<F, Item, Error> {
    fn new(subscriber_consumer: F) -> Self {
        Self {
            subscriber_consumer,
            phantom: PhantomData,
        }
    }
}

impl<F, Item, Error> core::Observable for FnObservable<F, Item, Error> {
    type Item = Item;
    type Error = Error;
}

impl<'o, F, Item, Error> core::LocalObservable<'o> for FnObservable<F, Item, Error>
where
    F: FnOnce(Box<dyn core::AutoUnsubscribeObserver<core::LocalSubscription, Item, Error> + 'o>),
    Item: 'o,
    Error: 'o,
{
    type Subscription = core::LocalSubscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Subscription, Self::Item, Self::Error> + 'o,
    {
        let mut observer = observer::local::AutoUnscubscribe::new(observer);
        observer.on_subscribe(observer.create_subscription());
        (self.subscriber_consumer)(Box::new(observer));
    }
}

impl<F, Item, Error> core::SharedObservable for FnObservable<F, Item, Error>
where
    F: FnOnce(
        Box<
            dyn core::AutoUnsubscribeObserver<core::SharedSubscription, Item, Error>
                + Send
                + Sync
                + 'static,
        >,
    ),
    Item: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Subscription = core::SharedSubscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::Observer<Self::Subscription, Self::Item, Self::Error> + Send + Sync + 'static,
    {
        let mut observer = observer::shared::AutoUnscubscribe::new(observer);
        observer.on_subscribe(observer.create_subscription());
        (self.subscriber_consumer)(Box::new(observer));
    }
}

pub fn create<F, Subscription, Item, Error>(observer_consumer: F) -> FnObservable<F, Item, Error> {
    FnObservable::new(observer_consumer)
}
