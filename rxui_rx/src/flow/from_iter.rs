use crate::core;
use crate::core::{FlowEmitter, IntoFlowEmitter, IntoSharedFlowEmitter};
use crate::marker;
use crate::subscription::*;
use crate::util;

#[doc(hidden)]
pub struct IntoIterFlow<IntoIter> {
    iterable: IntoIter,
}

impl<IntoIter> IntoIterFlow<IntoIter>
where
    IntoIter: IntoIterator,
{
    fn new(iterable: IntoIter) -> Self {
        Self { iterable }
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
    type Subscription = local::AccumulateSubscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + 'o,
    {
        let mut subscriber = subscriber.into_emitter();
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
    type Subscription = shared::AccumulateSubscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + Send + 'static,
    {
        let mut subscriber = subscriber.into_shared_emitter();
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

    fn into_flow(self) -> marker::Flow<Self::Flow> {
        marker::Flow::new(IntoIterFlow::new(self))
    }
}
