use crate::core;
use crate::flow;
use async_std::channel::{bounded, Receiver, Sender};
use async_std::sync::Mutex;
use async_trait::async_trait;
#[chronobreak]
use std::sync::atomic::{AtomicUsize, Ordering};
#[chronobreak]
use std::sync::{Arc, Weak};

#[operator(
    type = "flow",
    subscription = "OnBackpressureBufferSubscription<Subscription, Item, Error>"
)]
pub struct OnBackpressureBuffer {
    buffer_strategy: flow::BufferStrategy,
    buffer_capacity: usize,
}

pub struct OnBackpressureBufferSubscriber<Subscription, Item, Error> {
    data: Arc<Data<Subscription, Item, Error>>,
    buffer_strategy: flow::BufferStrategy,
}

type BoxedSubscriber<Subscription, Item, Error> = Box<
    dyn core::Subscriber<OnBackpressureBufferSubscription<Subscription, Item, Error>, Item, Error>
        + Send
        + 'static,
>;

pub struct Data<Subscription, Item, Error> {
    subscriber: Mutex<Option<BoxedSubscriber<Subscription, Item, Error>>>,
    requested: AtomicUsize,
    channel: (Sender<Item>, Receiver<Item>),
}

impl<Subscription, Item, Error> OnBackpressureBufferSubscriber<Subscription, Item, Error>
where
    Item: Send + 'static,
{
    pub fn new<Subscriber>(
        subscriber: Subscriber,
        buffer_strategy: flow::BufferStrategy,
        buffer_capacity: usize,
    ) -> Self
    where
        Subscriber: core::Subscriber<
                OnBackpressureBufferSubscription<Subscription, Item, Error>,
                Item,
                Error,
            > + Send
            + 'static,
    {
        let data = Arc::new(Data {
            subscriber: Mutex::new(Some(Box::new(subscriber))),
            requested: AtomicUsize::default(),
            channel: bounded(buffer_capacity),
        });
        Self {
            data,
            buffer_strategy,
        }
    }

    async fn add_to_queue(&self, item: Item) {
        if !self.data.channel.0.is_full() {
            self.data.channel.0.send(item).await.unwrap();
        } else {
            use flow::BufferStrategy::*;
            match self.buffer_strategy {
                Error => {
                    if let Some(mut subscriber) = self.data.subscriber.lock().await.take() {
                        subscriber.on_error(flow::Error::MissingBackpressure).await
                    };
                }
                DropOldest => {
                    self.data.channel.1.recv().await.unwrap();
                    self.data.channel.0.send(item).await.unwrap();
                }
                DropLatest => {}
            }
        }
    }
}

async fn drain<Subscription, Item, Error>(
    data: &Arc<Data<Subscription, Item, Error>>,
    subscriber: &mut BoxedSubscriber<Subscription, Item, Error>,
    mut requested: usize,
) {
    let mut emitted = 0;
    while emitted < requested {
        let item = if let Ok(item) = data.channel.1.recv().await {
            item
        } else {
            break;
        };
        subscriber.on_next(item).await;
        emitted += 1;
        // If the loop would finish, update the count of requested items as
        // on_next may have called request
        if emitted == requested {
            requested = data.requested.load(Ordering::Relaxed);
        }
    }
    data.requested.store(requested - emitted, Ordering::Relaxed);
}

#[async_trait]
impl<Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureBufferSubscriber<Subscription, Item, Error>
where
    Item: Send + 'static,
    Error: Send,
    Subscription: core::Subscription + Send + Sync + 'static,
{
    async fn on_subscribe(&mut self, subscription: Subscription) {
        let data = Arc::downgrade(&self.data);
        if let Some(subscriber) = self.data.subscriber.lock().await.as_mut() {
            subscription.request(usize::MAX).await;
            subscriber
                .on_subscribe(OnBackpressureBufferSubscription::new(subscription, data))
                .await
        }
    }

    async fn on_next(&mut self, item: Item) {
        let requested = self.data.requested.load(Ordering::Relaxed);
        self.add_to_queue(item).await;
        if requested > 0 {
            if let Some(ref mut subscriber) = *self.data.subscriber.lock().await {
                drain(&self.data, subscriber, requested).await;
            }
        }
    }

    async fn on_error(&mut self, error: flow::Error<Error>) {
        if let Some(subscriber) = self.data.subscriber.lock().await.as_mut() {
            subscriber.on_error(error).await
        }
    }

    async fn on_completed(&mut self) {
        if let Some(subscriber) = self.data.subscriber.lock().await.as_mut() {
            subscriber.on_completed().await
        }
    }
}

#[derive(new)]
pub struct OnBackpressureBufferSubscription<Upstream, Item, Error> {
    upstream: Upstream,
    data: Weak<Data<Upstream, Item, Error>>,
}

unsafe impl<Upstream, Item, Error> Sync
    for OnBackpressureBufferSubscription<Upstream, Item, Error>
{
}

#[async_trait]
impl<Upstream, Item, Error> core::Subscription
    for OnBackpressureBufferSubscription<Upstream, Item, Error>
where
    Upstream: core::Subscription + Send + Sync,
    Item: Send + 'static,
    Error: Send + 'static,
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

        let requested = data.requested.fetch_add(count, Ordering::Relaxed) + count;
        if requested > 0 {
            // This prevents more than one reentrant call of request is the
            // subscriber is borrowed mutably either here or in on_next
            if let Some(mut subscriber) = data.subscriber.try_lock() {
                if let Some(subscriber) = (&mut *subscriber).as_mut() {
                    drain(&data, subscriber, requested).await
                };
            }
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
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::Error, 5)
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
        assert_eq!(test_subscriber.items().await, vec![0, 1, 2]);
    }

    #[async_std::test]
    async fn upstream_error() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::Error, 1)
            .subscribe(test_subscriber.clone())
            .await;
        test_flow.emit(0).await;
        test_flow.emit_error(()).await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Error);
        assert_eq!(test_subscriber.items().await, vec![]);
    }

    #[async_std::test]
    async fn error_strategy() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::Error, 1)
            .subscribe(test_subscriber.clone())
            .await;
        test_flow.emit(0).await;
        test_flow.emit(1).await;
        test_subscriber.request_direct(1).await;
        test_flow.emit_completed().await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Error);
        assert_eq!(test_subscriber.items().await, vec![]);
    }

    #[async_std::test]
    async fn drop_oldest_strategy() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::DropOldest, 1)
            .subscribe(test_subscriber.clone())
            .await;
        test_flow.emit(0).await;
        test_flow.emit(1).await;
        test_subscriber.request_direct(1).await;
        test_flow.emit_completed().await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items().await, vec![1]);
    }

    #[async_std::test]
    async fn drop_latest_strategy() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::DropLatest, 1)
            .subscribe(test_subscriber.clone())
            .await;
        test_flow.emit(0).await;
        test_flow.emit(1).await;
        test_subscriber.request_direct(1).await;
        test_flow.emit_completed().await;
        assert_eq!(test_subscriber.status().await, SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items().await, vec![0]);
    }
}
