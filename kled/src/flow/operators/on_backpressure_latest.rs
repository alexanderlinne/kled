use crate::core;
use crate::flow;
use async_std::sync::Mutex;
use async_trait::async_trait;
use std::marker::PhantomData;
#[chronobreak]
use std::sync::atomic::{AtomicUsize, Ordering};
#[chronobreak]
use std::sync::{Arc, Weak};

#[operator(
    type = "flow",
    subscription = "OnBackpressureLatestSubscription<Subscription, Item, Error>"
)]
pub struct OnBackpressureLatest {}

type BoxedSubscriber<Subscription, Item, Error> = Box<
    dyn core::Subscriber<OnBackpressureLatestSubscription<Subscription, Item, Error>, Item, Error>
        + Send
        + 'static,
>;

pub struct OnBackpressureLatestSubscriber<Subscription, Item, Error> {
    subscriber: Arc<Mutex<BoxedSubscriber<Subscription, Item, Error>>>,
    data: Arc<Data<Item>>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

pub struct Data<Item> {
    requested: AtomicUsize,
    latest: Mutex<Option<Item>>,
}

impl<Subscription, Item, Error> OnBackpressureLatestSubscriber<Subscription, Item, Error>
where
    Item: Send + 'static,
{
    pub fn new<Subscriber>(subscriber: Subscriber) -> Self
    where
        Subscriber: core::Subscriber<
                OnBackpressureLatestSubscription<Subscription, Item, Error>,
                Item,
                Error,
            > + Send
            + 'static,
    {
        Self {
            subscriber: Arc::new(Mutex::new(Box::new(subscriber))),
            data: Arc::new(Data {
                requested: AtomicUsize::default(),
                latest: Mutex::new(None),
            }),
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureLatestSubscriber<Subscription, Item, Error>
where
    Item: Send,
    Error: Send,
    Subscription: core::Subscription + Send + Sync,
{
    async fn on_subscribe(&mut self, subscription: Subscription) {
        subscription.request(usize::MAX).await;
        let subscription = OnBackpressureLatestSubscription::new(
            subscription,
            Arc::downgrade(&self.subscriber),
            Arc::downgrade(&self.data),
        );
        self.subscriber
            .lock()
            .await
            .on_subscribe(subscription)
            .await;
    }

    async fn on_next(&mut self, item: Item) {
        if self.data.requested.load(Ordering::Relaxed) > 0 {
            let mut subscriber = self.subscriber.lock().await;
            *self.data.latest.lock().await = None;
            subscriber.on_next(item).await;
            self.data.requested.fetch_sub(1, Ordering::SeqCst);
        } else {
            *self.data.latest.lock().await = Some(item);
        }
    }

    async fn on_error(&mut self, error: flow::Error<Error>) {
        let mut subscriber = self.subscriber.lock().await;
        *self.data.latest.lock().await = None;
        subscriber.on_error(error).await;
    }

    async fn on_completed(&mut self) {
        let mut subscriber = self.subscriber.lock().await;
        *self.data.latest.lock().await = None;
        subscriber.on_completed().await;
    }
}

#[derive(new)]
pub struct OnBackpressureLatestSubscription<Upstream, Item, Error> {
    upstream: Upstream,
    subscriber: Weak<Mutex<BoxedSubscriber<Upstream, Item, Error>>>,
    data: Weak<Data<Item>>,
    phantom: PhantomData<Error>,
}

unsafe impl<Upstream, Item, Error> Sync
    for OnBackpressureLatestSubscription<Upstream, Item, Error>
{
}

#[async_trait]
impl<'o, Upstream, Item, Error> core::Subscription
    for OnBackpressureLatestSubscription<Upstream, Item, Error>
where
    Upstream: core::Subscription + Send + Sync + 'static,
    Item: Send,
    Error: Send,
{
    async fn cancel(&self) {
        self.upstream.cancel().await
    }

    async fn is_cancelled(&self) -> bool {
        self.upstream.is_cancelled().await
    }

    async fn request(&self, count: usize) {
        let data = match self.data.upgrade() {
            None => return,
            Some(data) => data,
        };

        let requested = data.requested.fetch_add(count, Ordering::SeqCst) + count;
        if requested > 0 {
            if let Some(subscriber) = self.subscriber.upgrade() {
                if let Some(mut subscriber) = subscriber.try_lock() {
                    let item = data.latest.lock().await.take();
                    if let Some(item) = item {
                        subscriber.on_next(item).await;
                        data.requested.fetch_sub(1, Ordering::SeqCst);
                    }
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::flow::*;
    use crate::prelude::*;
    use crate::subscriber::*;

    #[async_std::test]
    async fn basic() {
        let mut test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_latest()
            .subscribe(test_subscriber.clone())
            .await;
        test_flow.emit(0).await;
        test_flow.emit(1).await;
        test_subscriber.request_direct(1).await;
        test_flow.emit(2).await;
        test_subscriber.request_on_next(1).await;
        test_flow.emit(3).await;
        test_subscriber.request_direct(1).await;
        test_flow.emit(4).await;
        test_flow.emit_completed().await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items().await, vec![1, 3, 4]);
    }

    #[async_std::test]
    async fn error_case() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_latest()
            .subscribe(test_subscriber.clone())
            .await;
        test_flow.emit(0).await;
        test_flow.emit_error(()).await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Error);
        assert_eq!(test_subscriber.items().await, vec![]);
    }
}
