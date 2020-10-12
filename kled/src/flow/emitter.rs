use crate::core;
use crate::flow;
use crate::subscription::*;
use std::marker::PhantomData;

pub struct Emitter<Subscriber, Item, Error> {
    subscriber: Subscriber,
    stub: AccumulateSubscriptionStub,
    phantom: PhantomData<(Item, Error)>,
}

impl<Subscriber, Item, Error> Emitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<AccumulateSubscription, Item, Error>,
{
    fn new(mut subscriber: Subscriber) -> Self {
        let stub = AccumulateSubscriptionStub::default();
        subscriber.on_subscribe(stub.subscription());
        Self {
            subscriber,
            stub,
            phantom: PhantomData,
        }
    }

    pub fn on_next(&mut self, item: Item) {
        self.subscriber.on_next(item);
    }

    pub fn on_error(&mut self, error: Error) {
        self.subscriber.on_error(flow::Error::Upstream(error));
    }

    pub fn on_completed(&mut self) {
        self.subscriber.on_completed();
    }

    pub fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}

impl<Subscriber, Item, Error> From<Subscriber> for Emitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<AccumulateSubscription, Item, Error>,
{
    fn from(subscriber: Subscriber) -> Self {
        Emitter::new(subscriber)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::subscriber::*;

    #[test]
    fn basic() {
        let test_subscriber = TestSubscriber::new(1);
        let scheduler = scheduler::NewThreadScheduler::default();
        vec![0, 1, 2]
            .into_flow()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2]);
    }
}
