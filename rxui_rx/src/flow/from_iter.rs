use crate::core;
use crate::core::{FlowEmitter, IntoFlowEmitter, IntoSharedFlowEmitter};
use crate::flow;
use crate::marker;
use crate::util;

#[doc(hidden)]
pub struct IntoIterFlow<IntoIter> {
    strategy: flow::BackpressureStrategy,
    iterable: IntoIter,
}

impl<IntoIter> IntoIterFlow<IntoIter>
where
    IntoIter: IntoIterator,
{
    fn new(strategy: flow::BackpressureStrategy, iterable: IntoIter) -> Self {
        Self { strategy, iterable }
    }
}

impl<IntoIter> core::Flow for IntoIterFlow<IntoIter>
where
    IntoIter: IntoIterator,
{
    type Item = IntoIter::Item;
    type Error = util::Infallible;
}

impl<'o, IntoIter> core::LocalFlow<'o> for IntoIterFlow<IntoIter>
where
    IntoIter: IntoIterator,
    IntoIter::Item: 'o,
{
    type Subscription = Box<dyn core::Subscription + 'o>;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + 'o,
    {
        let mut subscriber = subscriber.into_emitter(self.strategy);
        for v in self.iterable.into_iter() {
            if !subscriber.is_cancelled() {
                subscriber.on_next(v);
            } else {
                break;
            }
        }
        if !subscriber.is_cancelled() {
            subscriber.on_completed();
        }
    }
}

impl<IntoIter> core::SharedFlow for IntoIterFlow<IntoIter>
where
    IntoIter: IntoIterator,
    IntoIter::Item: Send + 'static,
{
    type Subscription = Box<dyn core::Subscription + Send + 'static>;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + Send + 'static,
    {
        let mut subscriber = subscriber.into_shared_emitter(self.strategy);
        for v in self.iterable.into_iter() {
            if !subscriber.is_cancelled() {
                subscriber.on_next(v);
            } else {
                break;
            }
        }
        if !subscriber.is_cancelled() {
            subscriber.on_completed();
        }
    }
}

impl<IntoIter> core::IntoFlow for IntoIter
where
    IntoIter: IntoIterator,
{
    type Flow = IntoIterFlow<IntoIter>;

    fn into_flow(self, strategy: flow::BackpressureStrategy) -> marker::Flow<Self::Flow> {
        marker::Flow::new(IntoIterFlow::new(strategy, self))
    }
}
