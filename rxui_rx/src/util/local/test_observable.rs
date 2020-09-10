use crate::core;
use crate::core::{CancellableEmitter, Emitter};
use crate::emitter;
use std::cell::RefCell;
use std::rc::Rc;

pub struct TestObservable<'o, Item, Error> {
    data: Rc<RefCell<Data<'o, Item, Error>>>,
}

struct Data<'o, Item, Error> {
    emitter: Option<Box<dyn core::CancellableEmitter<Item, Error> + 'o>>,
}

impl<'o, Item, Error> Default for TestObservable<'o, Item, Error> {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new(Data { emitter: None })),
        }
    }
}

impl<'o, Item, Error> Clone for TestObservable<'o, Item, Error> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<'o, Item, Error> TestObservable<'o, Item, Error> {
    pub fn annotate_item_type(self, _: Item) -> Self {
        self
    }

    pub fn annotate_error_type(self, _: Error) -> Self {
        self
    }

    pub fn has_observer(&self) -> bool {
        self.data.borrow().emitter.is_some()
    }

    pub fn is_cancelled(&self) -> bool {
        assert!(self.has_observer());
        match self.data.borrow().emitter {
            Some(ref consumer) => consumer.is_cancelled(),
            None => panic!(),
        }
    }

    pub fn emit(&self, item: Item) {
        assert!(self.has_observer());
        match self.data.borrow_mut().emitter {
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
        match self.data.borrow_mut().emitter {
            Some(ref mut consumer) => consumer.on_error(error),
            None => panic!(),
        }
    }

    pub fn emit_on_completed(&self) {
        assert!(self.has_observer());
        match self.data.borrow_mut().emitter {
            Some(ref mut consumer) => consumer.on_completed(),
            None => panic!(),
        }
    }
}

impl<'o, Item, Error> core::LocalObservable<'o> for TestObservable<'o, Item, Error>
where
    Item: 'o,
    Error: 'o,
{
    type Cancellable = core::LocalCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + 'o,
    {
        assert!(!self.has_observer());
        self.data.borrow_mut().emitter =
            Some(Box::new(emitter::local::FromObserver::new(observer)));
    }
}

impl<'o, Item, Error> core::Observable for TestObservable<'o, Item, Error> {
    type Item = Item;
    type Error = Error;
}
