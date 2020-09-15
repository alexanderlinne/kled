use crate::core;
use crate::flow;
use crate::subscription::shared::*;
use std::marker::PhantomData;

pub struct MissingEmitter<Subscriber, Item, Error> {
    subscriber: Subscriber,
    stub: BoolSubscriptionStub,
    phantom: PhantomData<(Item, Error)>,
}

impl<Subscriber, Item, Error> MissingEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<BoolSubscription, Item, Error> + Send + 'static,
{
    pub fn new(mut subscriber: Subscriber) -> Self {
        let stub = BoolSubscriptionStub::default();
        subscriber.on_subscribe(stub.subscription());
        Self {
            subscriber,
            stub,
            phantom: PhantomData,
        }
    }
}

impl<Subscriber, Item, Error> core::FlowEmitter<Item, Error>
    for MissingEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<BoolSubscription, Item, Error> + Send + 'static,
{
    fn on_next(&mut self, item: Item) {
        self.subscriber.on_next(item);
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        self.subscriber.on_error(error);
    }

    fn on_completed(&mut self) {
        self.subscriber.on_completed();
    }

    fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }

    fn requested(&self) -> usize {
        self.stub.requested()
    }
}
