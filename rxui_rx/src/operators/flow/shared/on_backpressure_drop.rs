use crate::core;
use crate::flow;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(new, reactive_operator)]
pub struct FlowOnBackpressureDrop<Flow>
where
    Flow: core::SharedFlow,
    Flow::Item: Send,
    Flow::Error: Send,
{
    #[upstream(subscription = "OnBackpressureDropSubscription<Flow::Subscription, Flow::Error>")]
    flow: Flow,
}

pub struct OnBackpressureDropSubscriber<Subscription, Subscriber, Item, Error> {
    subscriber: Subscriber,
    requested: Arc<AtomicUsize>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<Subscription, Subscriber, Item, Error>
    OnBackpressureDropSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<OnBackpressureDropSubscription<Subscription, Error>, Item, Error>
        + Send
        + 'static,
{
    pub fn new(subscriber: Subscriber) -> Self {
        Self {
            subscriber,
            requested: Arc::new(AtomicUsize::default()),
            phantom: PhantomData,
        }
    }
}

impl<Subscription, Subscriber, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureDropSubscriber<Subscription, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<OnBackpressureDropSubscription<Subscription, Error>, Item, Error>
        + Send
        + 'static,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        let requested = self.requested.clone();
        self.subscriber
            .on_subscribe(OnBackpressureDropSubscription::new(subscription, requested));
    }

    fn on_next(&mut self, item: Item) {
        if self.requested.load(Ordering::Relaxed) > 0 {
            self.subscriber.on_next(item);
            self.requested.fetch_sub(1, Ordering::Relaxed);
        }
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        self.subscriber.on_error(error);
    }

    fn on_completed(&mut self) {
        self.subscriber.on_completed();
    }
}

#[derive(new)]
pub struct OnBackpressureDropSubscription<Upstream, Error> {
    upstream: Upstream,
    requested: Arc<AtomicUsize>,
    phantom: PhantomData<Error>,
}

unsafe impl<Upstream, Error> Sync for OnBackpressureDropSubscription<Upstream, Error> {}

impl<'o, Upstream, Error> core::Subscription for OnBackpressureDropSubscription<Upstream, Error>
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
    use crate::flow::shared::*;
    use crate::prelude::*;
    use crate::subscriber::shared::*;

    #[test]
    fn drop() {
        let test_subscriber = TestSubscriber::new(1);
        let scheduler = scheduler::NewThreadScheduler::default();
        vec![0, 1, 2]
            .into_shared_flow()
            .on_backpressure_drop()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
    }

    #[test]
    fn drop_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_drop()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }
}
