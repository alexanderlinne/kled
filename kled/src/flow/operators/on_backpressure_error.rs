use crate::core;
use crate::flow;
use std::marker::PhantomData;
#[chronobreak]
use std::sync::atomic::{AtomicUsize, Ordering};
#[chronobreak]
use std::sync::Arc;

#[derive(new)]
pub struct OnBackpressureError<Flow, Subscription, Item, Error>
where
    Flow: core::Flow<Subscription, Item, Error>,
{
    flow: Flow,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<Flow, Subscription, Item, Error>
    core::Flow<OnBackpressureErrorSubscription<Subscription>, Item, Error>
    for OnBackpressureError<Flow, Subscription, Item, Error>
where
    Flow: core::Flow<Subscription, Item, Error>,
    Subscription: core::Subscription + Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    fn subscribe<Downstream>(self, downstream: Downstream)
    where
        Downstream: core::Subscriber<OnBackpressureErrorSubscription<Subscription>, Item, Error>
            + Send
            + 'static,
    {
        self.flow
            .subscribe(OnBackpressureErrorSubscriber::new(downstream));
    }
}

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

impl<Subscription, Subscriber, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureErrorSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<OnBackpressureErrorSubscription<Subscription>, Item, Error>
        + Send
        + 'static,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        let requested = self.requested.clone();
        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_subscribe(OnBackpressureErrorSubscription::new(
                subscription,
                requested,
            ))
        };
    }

    fn on_next(&mut self, item: Item) {
        if let Some(ref mut subscriber) = self.subscriber {
            if self.requested.load(Ordering::Relaxed) > 0 {
                subscriber.on_next(item);
                self.requested.fetch_sub(1, Ordering::Relaxed);
            } else {
                subscriber.on_error(flow::Error::MissingBackpressure);
                self.subscriber = None
            }
        }
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        if let Some(ref mut subscriber) = self.subscriber {
            subscriber.on_error(error);
        }
    }

    fn on_completed(&mut self) {
        if let Some(ref mut subscriber) = self.subscriber {
            subscriber.on_completed();
        }
    }
}

#[derive(new)]
pub struct OnBackpressureErrorSubscription<Upstream> {
    upstream: Upstream,
    requested: Arc<AtomicUsize>,
}

unsafe impl<Upstream> Sync for OnBackpressureErrorSubscription<Upstream> {}

impl<'o, Upstream> core::Subscription for OnBackpressureErrorSubscription<Upstream>
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
        self.requested.fetch_add(count, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use crate::flow::*;
    use crate::prelude::*;
    use crate::subscriber::*;

    #[test]
    fn missing_backpressure() {
        let test_subscriber = TestSubscriber::default();
        vec![0, 1, 2]
            .into_flow()
            .on_backpressure_error()
            .subscribe(test_subscriber.clone());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
        matches!(
            test_subscriber.error(),
            Some(flow::Error::MissingBackpressure)
        );
    }

    #[test]
    fn upstream_error() {
        let test_subscriber = TestSubscriber::new(1);
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_error()
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![0]);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }

    #[test]
    fn basic() {
        let test_subscriber = TestSubscriber::new(1);
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_error()
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
        assert_eq!(test_subscriber.error(), None);
    }
}
