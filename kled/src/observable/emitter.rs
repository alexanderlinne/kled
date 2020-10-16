use crate::cancellable::*;
use crate::core;
use std::marker::PhantomData;

pub struct Emitter<Observer, Item, Error> {
    observer: Observer,
    stub: ArcCancellableStub,
    phantom: PhantomData<(Item, Error)>,
}

impl<Observer, Item, Error> Emitter<Observer, Item, Error>
where
    Observer: core::Observer<ArcCancellable, Item, Error>,
{
    fn new(mut observer: Observer) -> Self {
        let stub = ArcCancellableStub::default();
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

impl<Observer, Item, Error> From<Observer> for Emitter<Observer, Item, Error>
where
    Observer: core::Observer<ArcCancellable, Item, Error>,
{
    fn from(observer: Observer) -> Self {
        Emitter::new(observer)
    }
}
