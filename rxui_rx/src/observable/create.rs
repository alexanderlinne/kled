use crate::core;
use crate::core::Observer;
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

    fn subscribe<ObserverType>(self, observer: ObserverType)
    where
        ObserverType: core::Observer<Self::Item, Self::Error> + Send + Sync + 'static,
    {
        let mut subscriber = UnsubscribeMemorizingSubscriber::new(observer);
        subscriber.on_subscribe(Box::new(subscriber.create_subscription()));
        (self.subscriber_consumer)(Box::new(subscriber));
    }
}

pub fn create<F, Item, Error>(subscriber_consumer: F) -> FnObservable<F, Item, Error>
where
    F: FnOnce(Box<dyn core::Subscriber<Item, Error> + Send + Sync + 'static>),
{
    FnObservable::new(subscriber_consumer)
}
