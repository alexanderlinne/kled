use crate::subscription::*;
use crate::{core, flow, Never};
use async_trait::async_trait;

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

#[async_trait]
impl<IntoIter> core::Flow<ArcSubscription, IntoIter::Item, Never> for IntoIterFlow<IntoIter>
where
    IntoIter: IntoIterator + Send + 'static,
    IntoIter::Item: Send + 'static,
    IntoIter::IntoIter: Send,
{
    async fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<ArcSubscription, IntoIter::Item, Never> + Send + 'static,
    {
        let mut subscriber = flow::Emitter::from(subscriber).await;
        for v in self.iterable.into_iter() {
            if !subscriber.is_cancelled() {
                subscriber.on_next(v).await;
            } else {
                break;
            }
        }
        if !subscriber.is_cancelled() {
            subscriber.on_completed().await;
        }
    }
}

impl<IntoIter> core::IntoFlow<ArcSubscription, IntoIter::Item, Never> for IntoIter
where
    IntoIter: IntoIterator + Send + 'static,
    IntoIter::Item: Send,
    IntoIter::IntoIter: Send,
{
    type Flow = IntoIterFlow<IntoIter>;

    fn into_flow(self) -> Self::Flow {
        IntoIterFlow::new(self)
    }
}
