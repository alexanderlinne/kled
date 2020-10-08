use crate::cancellable::*;
use crate::core;
use crate::core::IntoObservableEmitter;
use crate::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TestObservable<Item, Error> {
    data: Arc<Mutex<Data<Item, Error>>>,
}

struct Data<Item, Error> {
    emitter: Option<Box<dyn core::ObservableEmitter<Item, Error> + Send + 'static>>,
}

impl<Item, Error> Default for TestObservable<Item, Error> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Data { emitter: None })),
        }
    }
}

impl<Item, Error> TestObservable<Item, Error> {
    pub fn has_observer(&self) -> bool {
        self.data.lock().emitter.is_some()
    }

    pub fn annotate_item_type(self, _: Item) -> Self {
        self
    }

    pub fn annotate_error_type(self, _: Error) -> Self {
        self
    }

    pub fn is_cancelled(&self) -> bool {
        assert!(self.has_observer());
        match self.data.lock().emitter {
            Some(ref consumer) => consumer.is_cancelled(),
            None => panic!(),
        }
    }

    pub fn emit(&self, item: Item) {
        assert!(self.has_observer());
        match self.data.lock().emitter {
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
        match self.data.lock().emitter {
            Some(ref mut consumer) => consumer.on_error(error),
            None => panic!(),
        }
    }

    pub fn emit_on_completed(&self) {
        assert!(self.has_observer());
        match self.data.lock().emitter {
            Some(ref mut consumer) => consumer.on_completed(),
            None => panic!(),
        }
    }
}

impl<Item, Error> core::Observable for TestObservable<Item, Error>
where
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Item = Item;
    type Error = Error;
    type Cancellable = BoolCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        assert!(!self.has_observer());
        self.data.lock().emitter = Some(Box::new(observer.into_emitter()));
    }
}