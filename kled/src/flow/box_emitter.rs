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
    pub async fn from<Subscriber>(mut subscriber: Subscriber) -> Self
    where
        Subscriber: core::Subscriber<ArcSubscription, Item, Error> + Send + 'static,
    {
        let stub = ArcSubscriptionStub::default();
        subscriber.on_subscribe(stub.subscription()).await;
        Self {
            subscriber: Box::new(subscriber),
            stub,
            phantom: PhantomData,
        }
    }

    pub async fn on_next(&mut self, item: Item) {
        self.subscriber.on_next(item).await;
    }

    pub async fn on_error(&mut self, error: Error) {
        self.subscriber.on_error(flow::Error::Upstream(error)).await;
    }

    pub async fn on_completed(&mut self) {
        self.subscriber.on_completed().await;
    }

    pub fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}
