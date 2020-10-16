use crate::core;
use crate::flow;
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

impl<IntoIter> core::Flow<ArcSubscription, IntoIter::Item, util::Infallible>
    for IntoIterFlow<IntoIter>
where
    IntoIter: IntoIterator,
    IntoIter::Item: Send + 'static,
{
    fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<ArcSubscription, IntoIter::Item, util::Infallible>
            + Send
            + 'static,
    {
        let mut subscriber = flow::Emitter::from(subscriber);
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

impl<IntoIter> core::IntoFlow<ArcSubscription, IntoIter::Item, util::Infallible> for IntoIter
where
    IntoIter: IntoIterator + 'static,
    IntoIter::Item: Send,
{
    type Flow = IntoIterFlow<IntoIter>;

    fn into_flow(self) -> Self::Flow {
        IntoIterFlow::new(self)
    }
}
