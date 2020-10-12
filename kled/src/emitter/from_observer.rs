use crate::cancellable::*;
use crate::core;
use std::marker::PhantomData;

pub struct FromObserver<Observer, Item, Error> {
    observer: Observer,
    stub: BoolCancellableStub,
    phantom: PhantomData<(Item, Error)>,
}

impl<Observer, Item, Error> FromObserver<Observer, Item, Error>
where
    Observer: core::Observer<BoolCancellable, Item, Error>,
{
    pub fn new(mut observer: Observer) -> Self {
        let stub = BoolCancellableStub::default();
        observer.on_subscribe(stub.cancellable());
        Self {
            observer,
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
