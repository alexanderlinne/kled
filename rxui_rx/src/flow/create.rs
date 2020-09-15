use crate::core;
use crate::core::{IntoFlowEmitter, IntoSharedFlowEmitter};
use crate::flow;
use crate::marker;
use crate::subscription;
use std::marker::PhantomData;

#[derive(Clone)]
#[doc(hidden)]
pub struct FlowCreate<F, Item, Error> {
    emitter_consumer: F,
    phantom: PhantomData<(Item, Error)>,
}

impl<F, Item, Error> FlowCreate<F, Item, Error> {
    pub fn new(emitter_consumer: F) -> Self {
        FlowCreate {
            emitter_consumer,
            phantom: PhantomData,
        }
    }
}

impl<'o, F, Item, Error> core::LocalFlow<'o> for FlowCreate<F, Item, Error>
where
    F: FnOnce(Box<dyn core::FlowEmitter<Item, Error> + 'o>),
    Item: 'o,
    Error: 'o,
{
    type Subscription = subscription::local::BoolSubscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + 'o,
    {
        let emitter = subscriber.into_emitter(flow::BackpressureStrategy::Missing);
        (self.emitter_consumer)(Box::new(emitter));
    }
}

impl<F, Item, Error> core::SharedFlow for FlowCreate<F, Item, Error>
where
    F: FnOnce(Box<dyn core::FlowEmitter<Item, Error> + Send>),
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Subscription = subscription::shared::BoolSubscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + Send + 'static,
    {
        let emitter = subscriber.into_shared_emitter(flow::BackpressureStrategy::Missing);
        (self.emitter_consumer)(Box::new(emitter));
    }
}

impl<F, Item, Error> core::Flow for FlowCreate<F, Item, Error> {
    type Item = Item;
    type Error = Error;
}

pub fn create<F, Item, Error>(emitter_consumer: F) -> marker::Flow<FlowCreate<F, Item, Error>> {
    marker::Flow::new(FlowCreate::new(emitter_consumer))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::util::local::*;

    #[test]
    fn create_missing() {
        let test_subscriber = TestSubscriber::default();
        flow::create(|mut emitter: Box<dyn FlowEmitter<i32, ()>>| {
            emitter.on_next(0);
            emitter.on_completed();
        })
        .subscribe(test_subscriber.clone());
        assert_eq!(test_subscriber.status(), ObserverStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
    }
}
