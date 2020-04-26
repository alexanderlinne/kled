use crate::core;
use crate::subscriber::*;
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

impl<F, Item, Error> core::Observable for FnObservable<F, Item, Error>
where
    F: FnOnce(Box<dyn core::Subscriber<Item, Error> + Send + Sync + 'static>),
    Item: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Item = Item;
    type Error = Error;
    type SubscriptionType = UnsubscribeMemorizingSubscription;

    fn subscribe<ObserverType>(self, observer: ObserverType) -> UnsubscribeMemorizingSubscription
    where
        ObserverType: core::Observer<Self::Item, Self::Error> + Send + Sync + 'static,
    {
        let subscriber = UnsubscribeMemorizingSubscriber::new(observer);
        let subscription = subscriber.create_subscription();
        (self.subscriber_consumer)(Box::new(subscriber));
        subscription
    }
}

pub fn create<F, Item, Error>(subscriber_consumer: F) -> FnObservable<F, Item, Error>
where
    F: FnOnce(Box<dyn core::Subscriber<Item, Error> + Send + Sync + 'static>),
{
    FnObservable::new(subscriber_consumer)
}
