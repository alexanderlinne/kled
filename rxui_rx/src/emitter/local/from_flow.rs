use crate::core;
use crate::flow;
use crate::subscription::local::*;

impl<'o, Subscriber, Item, Error> core::IntoFlowEmitter<'o, Item, Error> for Subscriber
where
    Subscriber: core::Subscriber<BoolSubscription, Item, Error> + 'o,
    Item: 'o,
    Error: 'o,
{
    type Emitter = Box<dyn core::FlowEmitter<Item, Error> + 'o>;

    fn into_emitter(self, strategy: flow::BackpressureStrategy) -> Self::Emitter {
        Box::new(match strategy {
            flow::BackpressureStrategy::Missing => super::flow::MissingEmitter::new(self),
            _ => unimplemented! {},
        })
    }
}
