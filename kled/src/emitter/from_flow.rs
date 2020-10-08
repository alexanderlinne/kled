use crate::core;
use crate::flow;
use crate::subscription::*;
use std::marker::PhantomData;

pub struct FromFlow<Subscriber, Item, Error> {
    subscriber: Subscriber,
    stub: AccumulateSubscriptionStub,
    phantom: PhantomData<(Item, Error)>,
}

impl<Subscriber, Item, Error> FromFlow<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<AccumulateSubscription, Item, Error> + Send + 'static,
{
    pub fn new(mut subscriber: Subscriber) -> Self {
        use crate::core::SubscriptionProvider;
        let stub = AccumulateSubscriptionStub::default();
        subscriber.on_subscribe(stub.subscription());
        Self {
            subscriber,
            stub,
            phantom: PhantomData,
        }
    }
}

impl<Subscriber, Item, Error> core::FlowEmitter<Item, Error> for FromFlow<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<AccumulateSubscription, Item, Error> + Send + 'static,
{
    fn on_next(&mut self, item: Item) {
        self.subscriber.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        self.subscriber.on_error(flow::Error::Upstream(error));
    }

    fn on_completed(&mut self) {
        self.subscriber.on_completed();
    }

    fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}

impl<Subscriber, Item, Error> core::IntoFlowEmitter<Item, Error> for Subscriber
where
    Subscriber: core::Subscriber<AccumulateSubscription, Item, Error> + Send + 'static,
{
    type Emitter = FromFlow<Subscriber, Item, Error>;

    fn into_emitter(self) -> Self::Emitter {
        FromFlow::new(self)
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
