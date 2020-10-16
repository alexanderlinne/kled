use crate::core;
use crate::flow;
use crate::subscription::*;
use std::marker::PhantomData;

pub struct BoxEmitter<Item, Error> {
    subscriber: Box<dyn core::Subscriber<ArcSubscription, Item, Error> + Send + 'static>,
    stub: ArcSubscriptionStub,
    phantom: PhantomData<(Item, Error)>,
}

impl<Item, Error> BoxEmitter<Item, Error> {
    pub fn from<Subscriber>(mut subscriber: Subscriber) -> Self
    where
        Subscriber: core::Subscriber<ArcSubscription, Item, Error> + Send + 'static,
    {
        let stub = ArcSubscriptionStub::default();
        subscriber.on_subscribe(stub.subscription());
        Self {
            subscriber: Box::new(subscriber),
            stub,
            phantom: PhantomData,
        }
    }

    pub fn on_next(&mut self, item: Item) {
        self.subscriber.on_next(item);
    }

    pub fn on_error(&mut self, error: Error) {
        self.subscriber.on_error(flow::Error::Upstream(error));
    }

    pub fn on_completed(&mut self) {
        self.subscriber.on_completed();
    }

    pub fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}
