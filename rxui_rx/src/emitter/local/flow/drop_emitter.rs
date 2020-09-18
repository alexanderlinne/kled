use crate::core;
use crate::flow;
use crate::subscription::local::*;
use std::cell::RefCell;
use std::marker::PhantomData;

pub struct DropEmitter<Subscriber, Item, Error> {
    subscriber: Subscriber,
    stub: BoolSubscriptionStub,
    requested: RefCell<usize>,
    phantom: PhantomData<(Item, Error)>,
}

impl<'o, Subscriber, Item, Error> DropEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<BoolSubscription, Item, Error> + 'o,
{
    pub fn new(mut subscriber: Subscriber) -> Self {
        let stub = BoolSubscriptionStub::default();
        subscriber.on_subscribe(stub.subscription());
        Self {
            subscriber,
            stub,
            requested: RefCell::new(0),
            phantom: PhantomData,
        }
    }

    fn update_requested(&self) -> usize {
        let mut requested = self.requested.borrow_mut();
        *requested += self.stub.get_and_reset_requested();
        *requested
    }
}

impl<'o, Subscriber, Item, Error> core::FlowEmitter<Item, Error>
    for DropEmitter<Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<BoolSubscription, Item, Error> + 'o,
{
    fn on_next(&mut self, item: Item) {
        let requested = self.update_requested();
        if requested > 0 {
            self.subscriber.on_next(item);
            *self.requested.borrow_mut() -= 1;
        }
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
}
