use crate::core;
use crate::flow;
use crate::subscription::local::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct DropEmitter<'o, Subscriber, Item, Error> {
    subscriber: Subscriber,
    stub: LambdaSubscriptionStub<'o>,
    requested: Rc<RefCell<usize>>,
    phantom: PhantomData<(Item, Error)>,
}

impl<'o, Subscriber, Item, Error> DropEmitter<'o, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<LambdaSubscription<'o>, Item, Error> + 'o,
{
    pub fn new(mut subscriber: Subscriber) -> Self {
        let requested = Rc::new(RefCell::new(0));
        let stub = Self::create_subscription(requested.clone());
        subscriber.on_subscribe(stub.subscription());
        Self {
            subscriber,
            stub,
            requested,
            phantom: PhantomData,
        }
    }

    fn create_subscription(requested: Rc<RefCell<usize>>) -> LambdaSubscriptionStub<'o> {
        LambdaSubscriptionStub::new(move |count: usize| {
            *requested.borrow_mut() += count;
        })
    }
}

impl<'o, Subscriber, Item, Error> core::FlowEmitter<Item, Error>
    for DropEmitter<'o, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<LambdaSubscription<'o>, Item, Error> + 'o,
{
    fn on_next(&mut self, item: Item) {
        if *self.requested.borrow() > 0 {
            self.subscriber.on_next(item);
            *self.requested.borrow_mut() -= 1;
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
