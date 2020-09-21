use crate::core;
use crate::flow;
use crate::subscription::local::*;
use std::marker::PhantomData;

pub struct DropEmitter<Subscriber, Item, Error> {
    subscriber: Subscriber,
    stub: AccumulateSubscriptionStub,
    requested: usize,
    phantom: PhantomData<(Item, Error)>,
}

impl<'o, Subscriber, Item, Error> DropEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<Box<dyn core::Subscription + 'o>, Item, Error> + 'o,
{
    pub fn new(mut subscriber: Subscriber) -> Self {
        let stub = AccumulateSubscriptionStub::default();
        subscriber.on_subscribe(Box::new(stub.subscription()));
        Self {
            subscriber,
            stub,
            requested: 0,
            phantom: PhantomData,
        }
    }

    fn update_request_count(&mut self) -> usize {
        self.requested += self.stub.get_and_reset_requested();
        self.requested
    }
}

impl<'o, Subscriber, Item, Error> core::FlowEmitter<Item, Error>
    for DropEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<Box<dyn core::Subscription + 'o>, Item, Error> + 'o,
{
    fn on_next(&mut self, item: Item) {
        let requested = self.update_request_count();
        if requested > 0 {
            self.subscriber.on_next(item);
            self.requested -= 1;
        }
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
