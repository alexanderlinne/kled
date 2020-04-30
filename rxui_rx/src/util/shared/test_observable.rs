use crate::consumer;
use crate::core;
use std::sync::{Arc, Mutex};

pub struct Data<Item, Error> {
    consumer: Option<Box<dyn core::CancellableConsumer<Item, Error> + Send + 'static>>,
}

pub struct TestObservable<Item, Error> {
    data: Arc<Mutex<Data<Item, Error>>>,
}

impl<'o, Item, Error> Default for TestObservable<Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data { consumer: None })),
        }
    }
}

impl<'o, Item, Error> Clone for TestObservable<Item, Error> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<Item, Error> TestObservable<Item, Error> {
    pub fn annotate_item_type(self, _: Item) -> Self {
        self
    }

    pub fn annotate_error_type(self, _: Error) -> Self {
        self
    }

    pub fn has_observer(&self) -> bool {
        self.data.lock().unwrap().consumer.is_some()
    }

    pub fn is_cancelled(&self) -> bool {
        assert!(self.has_observer());
        match self.data.lock().unwrap().consumer {
            Some(ref consumer) => consumer.is_cancelled(),
            None => panic!(),
        }
    }

    pub fn emit(&self, item: Item) {
        assert!(self.has_observer());
        match self.data.lock().unwrap().consumer {
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
        match self.data.lock().unwrap().consumer {
            Some(ref mut consumer) => consumer.on_error(error),
            None => panic!(),
        }
    }

    pub fn emit_on_completed(&self) {
        assert!(self.has_observer());
        match self.data.lock().unwrap().consumer {
            Some(ref mut consumer) => consumer.on_completed(),
            None => panic!(),
        }
    }
}

impl<Item, Error> core::SharedObservable for TestObservable<Item, Error>
where
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Subscription = core::SharedSubscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Subscription, Self::Item, Self::Error> + Send + 'static,
    {
        assert!(!self.has_observer());
        self.data.lock().unwrap().consumer =
            Some(Box::new(consumer::shared::AutoOnSubscribe::new(observer)));
    }
}

impl<Item, Error> core::Observable for TestObservable<Item, Error> {
    type Item = Item;
    type Error = Error;
}
