use crate::core;
use crate::emitter;
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
    F: FnOnce(Box<dyn core::UnsubscribableEmitter<Item, Error> + 'o>),
    Item: 'o,
    Error: 'o,
{
    type Observation = core::LocalObservation;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Observation, Self::Item, Self::Error> + 'o,
    {
        let observer = emitter::local::AutoOnSubscribeEmitter::new(observer);
        (self.subscriber_consumer)(Box::new(observer));
    }
}

impl<F, Item, Error> core::SharedObservable for FnObservable<F, Item, Error>
where
    F: FnOnce(Box<dyn core::UnsubscribableEmitter<Item, Error> + Send + Sync + 'static>),
    Item: Send + Sync + 'static,
    Error: Send + Sync + 'static,
{
    type Observation = core::SharedObservation;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::Observer<Self::Observation, Self::Item, Self::Error> + Send + Sync + 'static,
    {
        let observer = emitter::shared::AutoOnSubscribeEmitter::new(observer);
        (self.subscriber_consumer)(Box::new(observer));
    }
}

pub fn create<F, Subscription, Item, Error>(observer_consumer: F) -> FnObservable<F, Item, Error> {
    FnObservable::new(observer_consumer)
}
