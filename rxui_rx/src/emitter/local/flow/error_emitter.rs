use crate::core;
use crate::flow;
use crate::subscription::local::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct ErrorEmitter<'o, Subscriber, Item, Error> {
    subscriber: Option<Subscriber>,
    stub: LambdaSubscriptionStub<'o>,
    requested: Rc<RefCell<usize>>,
    phantom: PhantomData<(Item, Error)>,
}

impl<'o, Subscriber, Item, Error> ErrorEmitter<'o, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<LambdaSubscription<'o>, Item, Error> + 'o,
{
    pub fn new(mut subscriber: Subscriber) -> Self {
        let requested = Rc::new(RefCell::new(0));
        let stub = Self::create_subscription(requested.clone());
        subscriber.on_subscribe(stub.subscription());
        Self {
            subscriber: Some(subscriber),
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
    for ErrorEmitter<'o, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<LambdaSubscription<'o>, Item, Error> + 'o,
{
    fn on_next(&mut self, item: Item) {
        if let Some(ref mut subscriber) = self.subscriber {
            if *self.requested.borrow() > 0 {
                subscriber.on_next(item);
                *self.requested.borrow_mut() -= 1;
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
