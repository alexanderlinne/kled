use crate::core;
use crate::core::{FlowEmitter, IntoFlowEmitter};
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
    IntoIter::Item: Send + 'static,
{
    type Item = IntoIter::Item;
    type Error = util::Infallible;
    type Subscription = AccumulateSubscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + Send + 'static,
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

impl<IntoIter> core::IntoFlow for IntoIter
where
    IntoIter: IntoIterator + 'static,
    IntoIter::Item: Send,
{
    type Flow = IntoIterFlow<IntoIter>;

    fn into_flow(self) -> Self::Flow {
        IntoIterFlow::new(self)
    }
}
