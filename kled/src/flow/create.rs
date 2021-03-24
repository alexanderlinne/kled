use crate::core;
use crate::flow;
use crate::subscription::*;
use async_trait::async_trait;
use std::marker::PhantomData;

#[derive(new, Clone)]
#[doc(hidden)]
pub struct FlowCreate<F, Item, Error> {
    emitter_consumer: F,
    phantom: PhantomData<(Item, Error)>,
}

#[async_trait]
impl<F, Item, Error> core::Flow<ArcSubscription, Item, Error> for FlowCreate<F, Item, Error>
where
    F: FnOnce(flow::BoxEmitter<Item, Error>) + Send,
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<ArcSubscription, Item, Error> + Send + 'static,
    {
        (self.emitter_consumer)(flow::BoxEmitter::from(subscriber).await);
    }
}

pub fn create<F, Item, Error>(emitter_consumer: F) -> FlowCreate<F, Item, Error>
where
    F: FnOnce(flow::BoxEmitter<Item, Error>) + Send,
    Item: Send + 'static,
    Error: Send + 'static,
{
    FlowCreate::new(emitter_consumer)
}
