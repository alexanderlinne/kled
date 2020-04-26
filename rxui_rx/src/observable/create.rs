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

impl<F, Item, Error> core::Observable for FnObservable<F, Item, Error>
where
    F: FnOnce(Box<dyn core::AutoUnsubscribeObserver<Item, Error> + Send + Sync + 'static>),
    Item: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Item = Item;
    type Error = Error;

    fn subscribe<ObserverType>(self, observer: ObserverType)
    where
        ObserverType: core::Observer<Self::Item, Self::Error> + Send + Sync + 'static,
    {
        let mut observer = observer::AutoUnscubscribe::new(observer);
        observer.on_subscribe(Box::new(observer.create_subscription()));
        (self.subscriber_consumer)(Box::new(observer));
    }
}

pub fn create<F, Item, Error>(subscriber_consumer: F) -> FnObservable<F, Item, Error>
where
    F: FnOnce(Box<dyn core::AutoUnsubscribeObserver<Item, Error> + Send + Sync + 'static>),
{
    FnObservable::new(subscriber_consumer)
}
