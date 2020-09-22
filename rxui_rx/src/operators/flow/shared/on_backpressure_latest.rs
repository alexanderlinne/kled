use crate::core;
use crate::flow;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

#[derive(new, reactive_operator)]
pub struct FlowOnBackpressureLatest<Flow>
where
    Flow: core::SharedFlow,
    Flow::Item: Send,
    Flow::Error: Send,
{
    #[upstream(
        downstream = "OnBackpressureLatestSubscriber",
        subscription = "OnBackpressureLatestSubscription<Flow::Subscription, Flow::Item, Flow::Error>"
    )]
    flow: Flow,
}

pub struct OnBackpressureLatestSubscriber<Subscription, Subscriber, Item, Error> {
    subscriber: Arc<Mutex<Subscriber>>,
    data: Arc<Data<Item>>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

pub struct Data<Item> {
    requested: AtomicUsize,
    latest: Mutex<Option<Item>>,
}

impl<Subscription, Subscriber, Item, Error>
    OnBackpressureLatestSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<OnBackpressureLatestSubscription<Subscription, Item, Error>, Item, Error>
        + Send
        + 'static,
    Item: Send + 'static,
{
    pub fn new(subscriber: Subscriber) -> Self {
        let subscriber = Arc::new(Mutex::new(subscriber));
        let data = Arc::new(Data {
            requested: AtomicUsize::default(),
            latest: Mutex::new(None),
        });
        Self {
            subscriber,
            data,
            phantom: PhantomData,
        }
    }
}

impl<Subscription, Subscriber, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureLatestSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<OnBackpressureLatestSubscription<Subscription, Item, Error>, Item, Error>
        + Send
        + 'static,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        let subscription = OnBackpressureLatestSubscription::new(
            subscription,
            Arc::downgrade(&self.subscriber).clone(),
            Arc::downgrade(&self.data),
        );
        self.subscriber.lock().unwrap().on_subscribe(subscription);
    }

    fn on_next(&mut self, item: Item) {
        if self.data.requested.load(Ordering::Relaxed) > 0 {
            let mut subscriber = self.subscriber.lock().unwrap();
            *self.data.latest.lock().unwrap() = None;
            subscriber.on_next(item);
            self.data.requested.fetch_sub(1, Ordering::Relaxed);
        } else {
            *self.data.latest.lock().unwrap() = Some(item);
        }
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        let mut subscriber = self.subscriber.lock().unwrap();
        *self.data.latest.lock().unwrap() = None;
        subscriber.on_error(error);
    }

    fn on_completed(&mut self) {
        let mut subscriber = self.subscriber.lock().unwrap();
        *self.data.latest.lock().unwrap() = None;
        subscriber.on_completed();
    }
}

#[derive(new)]
pub struct OnBackpressureLatestSubscription<Upstream, Item, Error> {
    upstream: Upstream,
    subscriber: Weak<Mutex<dyn core::Subscriber<Self, Item, Error> + Send + 'static>>,
    data: Weak<Data<Item>>,
    phantom: PhantomData<Error>,
}

unsafe impl<Upstream, Item, Error> Sync
    for OnBackpressureLatestSubscription<Upstream, Item, Error>
{
}

impl<'o, Upstream, Item, Error> core::Subscription
    for OnBackpressureLatestSubscription<Upstream, Item, Error>
where
    Upstream: core::Subscription + Send + Sync + 'static,
{
    fn cancel(&self) {
        self.upstream.cancel()
    }

    fn is_cancelled(&self) -> bool {
        self.upstream.is_cancelled()
    }

    fn request(&self, count: usize) {
        let data = match self.data.upgrade() {
            None => return,
            Some(data) => data,
        };

        let requested = data.requested.fetch_add(count, Ordering::Relaxed) + count;
        if requested > 0 {
            self.subscriber.upgrade().map(|subscriber| {
                if let Ok(mut subscriber) = subscriber.try_lock() {
                    let item = data.latest.lock().unwrap().take();
                    if let Some(item) = item {
                        subscriber.on_next(item);
                        data.requested.fetch_sub(1, Ordering::Relaxed);
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::util::shared::*;

    #[test]
    fn basic() {
        let mut test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_latest()
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit(1);
        test_subscriber.request_direct(1);
        test_flow.emit(2);
        test_subscriber.request_on_next(1);
        test_flow.emit(3);
        test_subscriber.request_direct(1);
        test_flow.emit(4);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![1, 3, 4]);
    }

    #[test]
    fn error_case() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_latest()
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
    }
}
