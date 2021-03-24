use crate::core;
use crate::flow;
use async_trait::async_trait;
use std::marker::PhantomData;
#[chronobreak]
use std::sync::atomic::{AtomicUsize, Ordering};
#[chronobreak]
use std::sync::Arc;

#[operator(type = "flow", subscription = "OnBackpressureErrorSubscription<Subscription>")]
pub struct OnBackpressureError {}

pub struct OnBackpressureErrorSubscriber<Subscription, Subscriber, Item, Error> {
    subscriber: Option<Subscriber>,
    requested: Arc<AtomicUsize>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<Subscription, Subscriber, Item, Error>
    OnBackpressureErrorSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<OnBackpressureErrorSubscription<Subscription>, Item, Error>
        + Send
        + 'static,
{
    pub fn new(subscriber: Subscriber) -> Self {
        Self {
            subscriber: Some(subscriber),
            requested: Arc::new(AtomicUsize::default()),
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Subscription, Subscriber, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureErrorSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<OnBackpressureErrorSubscription<Subscription>, Item, Error>
        + Send
        + 'static,
    Item: Send,
    Error: Send,
    Subscription: core::Subscription + Send + Sync,
{
    async fn on_subscribe(&mut self, subscription: Subscription) {
        let requested = self.requested.clone();
        if let Some(subscriber) = self.subscriber.as_mut() {
            subscription.request(usize::MAX).await;
            subscriber.on_subscribe(OnBackpressureErrorSubscription::new(
                subscription,
                requested,
            )).await
        };
    }

    async fn on_next(&mut self, item: Item) {
        if let Some(ref mut subscriber) = self.subscriber {
            if self.requested.load(Ordering::Relaxed) > 0 {
                subscriber.on_next(item).await;
                self.requested.fetch_sub(1, Ordering::Relaxed);
            } else {
                subscriber.on_error(flow::Error::MissingBackpressure).await;
                self.subscriber = None
            }
        }
    }

    async fn on_error(&mut self, error: flow::Error<Error>) {
        if let Some(ref mut subscriber) = self.subscriber {
            subscriber.on_error(error).await;
        }
    }

    async fn on_completed(&mut self) {
        if let Some(ref mut subscriber) = self.subscriber {
            subscriber.on_completed().await;
        }
    }
}

#[derive(new)]
pub struct OnBackpressureErrorSubscription<Upstream> {
    upstream: Upstream,
    requested: Arc<AtomicUsize>,
}

#[async_trait]
impl<'o, Upstream> core::Subscription for OnBackpressureErrorSubscription<Upstream>
where
    Upstream: core::Subscription + Send + Sync + 'static,
{
    async fn cancel(&self) {
        self.upstream.cancel().await
    }

    async fn is_cancelled(&self) -> bool {
        self.upstream.is_cancelled().await
    }

    async fn request(&self, count: usize) {
        self.requested.fetch_add(count, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use crate::flow::*;
    use crate::prelude::*;
    use crate::subscriber::*;

    #[async_std::test]
    async fn missing_backpressure() {
        let test_subscriber = TestSubscriber::default();
        vec![0, 1, 2]
            .into_flow()
            .on_backpressure_error()
            .subscribe(test_subscriber.clone()).await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Error);
        assert_eq!(test_subscriber.items().await, vec![]);
        matches!(
            test_subscriber.error().await,
            Some(flow::Error::MissingBackpressure)
        );
    }

    #[async_std::test]
    async fn upstream_error() {
        let test_subscriber = TestSubscriber::new(1);
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_error()
            .subscribe(test_subscriber.clone()).await;
        test_flow.emit(0).await;
        test_flow.emit_error(()).await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Error);
        assert_eq!(test_subscriber.items().await, vec![0]);
        assert_eq!(test_subscriber.error().await, Some(flow::Error::Upstream(())));
    }

    #[async_std::test]
    async fn basic() {
        let test_subscriber = TestSubscriber::new(1);
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_error()
            .subscribe(test_subscriber.clone()).await;
        test_flow.emit(0).await;
        test_flow.emit_completed().await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items().await, vec![0]);
        assert_eq!(test_subscriber.error().await, None);
    }
}
