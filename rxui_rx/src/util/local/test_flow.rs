use crate::core;
use crate::core::IntoFlowEmitter;
use crate::flow;
use crate::marker;
use crate::subscription::local::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct TestFlow<'o, Item, Error> {
    data: Rc<RefCell<Data<'o, Item, Error>>>,
}

struct Data<'o, Item, Error> {
    strategy: flow::BackpressureStrategy,
    emitter: Option<Box<dyn core::FlowEmitter<Item, Error> + 'o>>,
}

impl<'o, Item, Error> TestFlow<'o, Item, Error> {
    pub fn default() -> marker::Flow<Self> {
        marker::Flow::new(Self {
            data: Rc::new(RefCell::new(Data {
                strategy: flow::BackpressureStrategy::Missing,
                emitter: None,
            })),
        })
    }

    pub fn new(strategy: flow::BackpressureStrategy) -> marker::Flow<Self> {
        marker::Flow::new(Self {
            data: Rc::new(RefCell::new(Data {
                strategy,
                emitter: None,
            })),
        })
    }
}

impl<'o, Item, Error> TestFlow<'o, Item, Error> {
    pub fn has_observer(&self) -> bool {
        self.data.borrow().emitter.is_some()
    }
}

impl<'o, Item, Error> marker::Flow<TestFlow<'o, Item, Error>> {
    pub fn annotate_item_type(self, _: Item) -> Self {
        self
    }

    pub fn annotate_error_type(self, _: Error) -> Self {
        self
    }

    pub fn has_observer(&self) -> bool {
        self.actual.has_observer()
    }

    pub fn is_cancelled(&self) -> bool {
        assert!(self.has_observer());
        match self.actual.data.borrow().emitter {
            Some(ref consumer) => consumer.is_cancelled(),
            None => panic!(),
        }
    }

    pub fn emit(&self, item: Item) {
        assert!(self.has_observer());
        match self.actual.data.borrow_mut().emitter {
            Some(ref mut consumer) => consumer.on_next(item),
            None => panic!(),
        }
    }

    pub fn emit_all<IntoIter>(&self, into_iter: IntoIter)
    where
        IntoIter: IntoIterator<Item = Item>,
    {
        for value in into_iter.into_iter() {
            self.emit(value);
        }
    }

    pub fn emit_error(&self, error: Error) {
        assert!(self.has_observer());
        match self.actual.data.borrow_mut().emitter {
            Some(ref mut consumer) => consumer.on_error(error),
            None => panic!(),
        }
    }

    pub fn emit_on_completed(&self) {
        assert!(self.has_observer());
        match self.actual.data.borrow_mut().emitter {
            Some(ref mut consumer) => consumer.on_completed(),
            None => panic!(),
        }
    }
}

impl<'o, Item, Error> core::LocalFlow<'o> for TestFlow<'o, Item, Error>
where
    Item: 'o,
    Error: 'o,
{
    type Subscription = BoolSubscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + 'o,
    {
        assert!(!self.has_observer());
        let mut data = self.data.borrow_mut();
        data.emitter = Some(Box::new(subscriber.into_emitter(data.strategy)));
    }
}

impl<'o, Item, Error> core::Flow for TestFlow<'o, Item, Error> {
    type Item = Item;
    type Error = Error;
}
