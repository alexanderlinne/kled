use crate::core;
use crate::flow;
use crate::subscription::local::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct LatestEmitter<'o, Subscriber, Item, Error> {
    subscriber: Rc<RefCell<Subscriber>>,
    stub: LambdaSubscriptionStub<'o>,
    data: Rc<RefCell<Data<Item>>>,
    phantom: PhantomData<Error>,
}

pub struct Data<Item> {
    requested: usize,
    latest: Option<Item>,
}

impl<'o, Subscriber, Item, Error> LatestEmitter<'o, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<Box<dyn core::Subscription + 'o>, Item, Error> + 'o,
    Item: 'o,
{
    pub fn new(subscriber: Subscriber) -> Self {
        let subscriber = Rc::new(RefCell::new(subscriber));
        let data = Rc::new(RefCell::new(Data {
            requested: 0,
            latest: None,
        }));
        let stub = Self::create_subscription(subscriber.clone(), data.clone());
        subscriber
            .borrow_mut()
            .on_subscribe(Box::new(stub.subscription()));
        Self {
            subscriber,
            stub,
            data,
            phantom: PhantomData,
        }
    }

    fn create_subscription(
        subscriber: Rc<RefCell<Subscriber>>,
        data: Rc<RefCell<Data<Item>>>,
    ) -> LambdaSubscriptionStub<'o> {
        LambdaSubscriptionStub::new(move |count: usize| {
            let mut data_ref = data.borrow_mut();
            data_ref.requested += count;
            let requested = data_ref.requested;
            drop(data_ref);

            if requested > 0 {
                let item = data.borrow_mut().latest.take();
                // this allows for only one further reentrant call of request
                // because another item cannot be produced here in synchronous
                // code (unless the subscriber is the producer, which is not a
                // sensible use case)
                if let Some(item) = item {
                    subscriber.borrow_mut().on_next(item);
                    data.borrow_mut().requested -= 1;
                }
            }
        })
    }
}

impl<'o, Subscriber, Item, Error> core::FlowEmitter<Item, Error>
    for LatestEmitter<'o, Subscriber, Item, Error>
where
    Subscriber: core::Subscriber<Box<dyn core::Subscription + 'o>, Item, Error> + 'o,
{
    fn on_next(&mut self, item: Item) {
        // data must be borrowed individually here to allow on_next to call
        // request safely (which borrows data mutably)
        if self.data.borrow().requested > 0 {
            self.subscriber.borrow_mut().on_next(item);
            self.data.borrow_mut().requested -= 1;
        } else {
            self.data.borrow_mut().latest = Some(item);
        }
    }

    fn on_error(&mut self, error: Error) {
        self.subscriber
            .borrow_mut()
            .on_error(flow::Error::Upstream(error));
    }

    fn on_completed(&mut self) {
        self.subscriber.borrow_mut().on_completed();
    }

    fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}
