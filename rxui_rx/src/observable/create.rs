use crate::consumer;
use crate::core;
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
    F: FnOnce(Box<dyn core::CancellableConsumer<Item, Error> + 'o>),
    Item: 'o,
    Error: 'o,
{
    type Cancellable = core::LocalCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + 'o,
    {
        let observer = consumer::local::AutoOnSubscribe::new(observer);
        (self.subscriber_consumer)(Box::new(observer));
    }
}

impl<F, Item, Error> core::SharedObservable for FnObservable<F, Item, Error>
where
    F: FnOnce(Box<dyn core::CancellableConsumer<Item, Error> + Send + 'static>),
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Cancellable = core::SharedCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        let observer = consumer::shared::AutoOnSubscribe::new(observer);
        (self.subscriber_consumer)(Box::new(observer));
    }
}

pub fn create<F, Item, Error>(observer_consumer: F) -> FnObservable<F, Item, Error> {
    FnObservable::new(observer_consumer)
}
