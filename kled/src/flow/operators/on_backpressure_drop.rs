use crate::core;
use crate::flow;
use async_trait::async_trait;
use std::marker::PhantomData;
#[chronobreak]
use std::sync::atomic::{AtomicUsize, Ordering};
#[chronobreak]
use std::sync::Arc;

#[operator(
    type = "flow",
    subscription = "OnBackpressureDropSubscription<Subscription>"
)]
pub struct OnBackpressureDrop {}

pub struct OnBackpressureDropSubscriber<Subscription, Subscriber, Item, Error> {
    subscriber: Subscriber,
    requested: Arc<AtomicUsize>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<Subscription, Subscriber, Item, Error>
    OnBackpressureDropSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<OnBackpressureDropSubscription<Subscription>, Item, Error>,
{
    pub fn new(subscriber: Subscriber) -> Self {
        Self {
            subscriber,
            requested: Arc::new(AtomicUsize::default()),
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Subscription, Subscriber, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureDropSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<OnBackpressureDropSubscription<Subscription>, Item, Error> + Send,
    Item: Send,
    Error: Send,
    Subscription: core::Subscription + Send + Sync,
{
    async fn on_subscribe(&mut self, subscription: Subscription) {
        let requested = self.requested.clone();
        subscription.request(usize::MAX).await;
        self.subscriber
            .on_subscribe(OnBackpressureDropSubscription::new(subscription, requested))
            .await;
    }

    async fn on_next(&mut self, item: Item) {
        if self.requested.load(Ordering::Relaxed) > 0 {
            self.subscriber.on_next(item).await;
            self.requested.fetch_sub(1, Ordering::Relaxed);
        }
    }

    async fn on_error(&mut self, error: flow::Error<Error>) {
        self.subscriber.on_error(error).await;
    }

    async fn on_completed(&mut self) {
        self.subscriber.on_completed().await;
    }
}

#[derive(new)]
pub struct OnBackpressureDropSubscription<Upstream> {
    upstream: Upstream,
    requested: Arc<AtomicUsize>,
}

#[async_trait]
impl<Upstream> core::Subscription for OnBackpressureDropSubscription<Upstream>
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
    async fn drop_completed() {
        let test_subscriber = TestSubscriber::new(1);
        vec![0, 1, 2]
            .into_flow()
            .on_backpressure_drop()
            .subscribe(test_subscriber.clone())
            .await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items().await, vec![0]);
    }

    #[async_std::test]
    async fn drop_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_drop()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone())
            .await;
        test_flow.emit(0).await;
        test_flow.emit_error(()).await;
        scheduler.join();
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Error);
        assert_eq!(
            test_subscriber.error().await,
            Some(flow::Error::Upstream(()))
        );
    }
}
