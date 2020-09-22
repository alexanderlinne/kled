use crate::core;
use crate::flow;
use crate::subscription::shared::*;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct ErrorEmitter<Subscriber, Item, Error> {
    subscriber: Option<Subscriber>,
    stub: LambdaSubscriptionStub,
    requested: Arc<AtomicUsize>,
    phantom: PhantomData<(Item, Error)>,
}

impl<Subscriber, Item, Error> ErrorEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<LambdaSubscription, Item, Error> + Send + 'static,
{
    pub fn new(mut subscriber: Subscriber) -> Self {
        let requested = Arc::new(AtomicUsize::default());
        let stub = Self::create_subscription(requested.clone());
        subscriber.on_subscribe(stub.subscription());
        Self {
            subscriber: Some(subscriber),
            stub,
            requested,
            phantom: PhantomData,
        }
    }

    fn create_subscription(requested: Arc<AtomicUsize>) -> LambdaSubscriptionStub {
        LambdaSubscriptionStub::new(move |count: usize| {
            requested.fetch_add(count, Ordering::Relaxed);
        })
    }
}

impl<Subscriber, Item, Error> core::FlowEmitter<Item, Error>
    for ErrorEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<LambdaSubscription, Item, Error> + Send + 'static,
{
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

    fn on_error(&mut self, error: Error) {
        if let Some(ref mut subscriber) = self.subscriber {
            subscriber.on_error(flow::Error::Upstream(error));
        }
    }

    fn on_completed(&mut self) {
        if let Some(ref mut subscriber) = self.subscriber {
            subscriber.on_completed();
        }
    }

    fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}
