use crate::core;
use crate::flow;
use crate::subscription::local::*;
use std::marker::PhantomData;

pub struct FromFlow<Subscriber, Item, Error> {
    subscriber: Subscriber,
    stub: AccumulateSubscriptionStub,
    phantom: PhantomData<(Item, Error)>,
}

impl<'o, Subscriber, Item, Error> FromFlow<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<AccumulateSubscription, Item, Error> + 'o,
{
    pub fn new(mut subscriber: Subscriber) -> Self {
        let stub = AccumulateSubscriptionStub::default();
        subscriber.on_subscribe(stub.subscription());
        Self {
            subscriber,
            stub,
            phantom: PhantomData,
        }
    }
}

impl<'o, Subscriber, Item, Error> core::FlowEmitter<Item, Error>
    for FromFlow<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<AccumulateSubscription, Item, Error> + 'o,
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

impl<'o, Subscriber, Item, Error> core::IntoFlowEmitter<'o, Item, Error> for Subscriber
where
    Subscriber: core::Subscriber<AccumulateSubscription, Item, Error> + 'o,
{
    type Emitter = FromFlow<Subscriber, Item, Error>;

    fn into_emitter(self) -> Self::Emitter {
        FromFlow::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::flow::local::*;
    use crate::prelude::*;
    use crate::subscriber::local::*;

    #[test]
    fn basic() {
        let test_subscriber = TestSubscriber::new(1);
        vec![0, 1, 2].into_flow().subscribe(test_subscriber.clone());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2]);
    }

    #[test]
    fn error_case() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow.clone().subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }
}
