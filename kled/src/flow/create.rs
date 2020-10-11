use crate::core;
use crate::core::IntoFlowEmitter;
use crate::subscription::*;
use std::marker::PhantomData;

#[derive(new, Clone)]
#[doc(hidden)]
pub struct FlowCreate<F, Item, Error> {
    emitter_consumer: F,
    phantom: PhantomData<(Item, Error)>,
}

impl<F, Item, Error> core::Flow<AccumulateSubscription, Item, Error> for FlowCreate<F, Item, Error>
where
    F: FnOnce(Box<dyn core::FlowEmitter<Item, Error> + Send>),
    Item: Send + 'static,
    Error: Send + 'static,
{
    fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<AccumulateSubscription, Item, Error> + Send + 'static,
    {
        let emitter = subscriber.into_emitter();
        (self.emitter_consumer)(Box::new(emitter));
    }
}

pub fn create<F, Item, Error>(emitter_consumer: F) -> FlowCreate<F, Item, Error> {
    FlowCreate::new(emitter_consumer)
}
