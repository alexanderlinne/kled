use crate::cancellable::*;
use crate::core;
use std::marker::PhantomData;

pub struct BoxEmitter<Item, Error> {
    observer: Box<dyn core::Observer<BoolCancellable, Item, Error> + Send + 'static>,
    stub: BoolCancellableStub,
    phantom: PhantomData<(Item, Error)>,
}

impl<Item, Error> BoxEmitter<Item, Error> {
    pub fn from<Observer>(mut observer: Observer) -> Self
    where
        Observer: core::Observer<BoolCancellable, Item, Error> + Send + 'static,
    {
        let stub = BoolCancellableStub::default();
        observer.on_subscribe(stub.cancellable());
        Self {
            observer: Box::new(observer),
            stub,
            phantom: PhantomData,
        }
    }

    pub fn on_next(&mut self, item: Item) {
        self.observer.on_next(item);
    }

    pub fn on_error(&mut self, error: Error) {
        self.observer.on_error(error);
    }

    pub fn on_completed(&mut self) {
        self.observer.on_completed();
    }

    pub fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}
