use crate::core;
use crate::core::IntoSharedFlowEmitter;
use crate::marker;
use crate::subscription::shared::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TestFlow<Item, Error> {
    data: Arc<Mutex<Data<Item, Error>>>,
}

struct Data<Item, Error> {
    emitter: Option<Box<dyn core::FlowEmitter<Item, Error> + Send + 'static>>,
}

impl<Item, Error> TestFlow<Item, Error> {
    pub fn default() -> marker::Shared<marker::Flow<Self>> {
        marker::Shared::new(marker::Flow::new(Self {
            data: Arc::new(Mutex::new(Data { emitter: None })),
        }))
    }
}

impl<Item, Error> TestFlow<Item, Error> {
    pub fn has_observer(&self) -> bool {
        self.data.lock().unwrap().emitter.is_some()
    }
}

impl<Item, Error> marker::Shared<marker::Flow<TestFlow<Item, Error>>> {
    pub fn annotate_item_type(self, _: Item) -> Self {
        self
    }

    pub fn annotate_error_type(self, _: Error) -> Self {
        self
    }

    pub fn has_observer(&self) -> bool {
        self.actual.actual.has_observer()
    }

    pub fn is_cancelled(&self) -> bool {
        assert!(self.has_observer());
        match self.actual.actual.data.lock().unwrap().emitter {
            Some(ref consumer) => consumer.is_cancelled(),
            None => panic!(),
        }
    }

    pub fn emit(&self, item: Item) {
        assert!(self.has_observer());
        match self.actual.actual.data.lock().unwrap().emitter {
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
        match self.actual.actual.data.lock().unwrap().emitter {
            Some(ref mut consumer) => consumer.on_error(error),
            None => panic!(),
        }
    }

    pub fn emit_completed(&self) {
        assert!(self.has_observer());
        match self.actual.actual.data.lock().unwrap().emitter {
            Some(ref mut consumer) => consumer.on_completed(),
            None => panic!(),
        }
    }
}

impl<Item, Error> core::SharedFlow for TestFlow<Item, Error>
where
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Subscription = AccumulateSubscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + Send + 'static,
    {
        assert!(!self.has_observer());
        let mut data = self.data.lock().unwrap();
        data.emitter = Some(Box::new(subscriber.into_shared_emitter()));
    }
}

impl<Item, Error> core::Flow for TestFlow<Item, Error> {
    type Item = Item;
    type Error = Error;
}
