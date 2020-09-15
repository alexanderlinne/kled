use crate::core;
use crate::flow;
use crate::subscription::shared::*;

impl<Subscriber, Item, Error> core::IntoSharedFlowEmitter<Item, Error> for Subscriber
where
    Subscriber: core::Subscriber<BoolSubscription, Item, Error> + Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Emitter = Box<dyn core::FlowEmitter<Item, Error> + Send + 'static>;

    fn into_shared_emitter(self, strategy: flow::BackpressureStrategy) -> Self::Emitter {
        Box::new(match strategy {
            flow::BackpressureStrategy::Missing => super::flow::MissingEmitter::new(self),
            _ => unimplemented! {},
        })
    }
}
