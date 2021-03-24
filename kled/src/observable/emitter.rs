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
    pub async fn from(mut observer: Observer) -> Self {
        let stub = ArcCancellableStub::default();
        observer.on_subscribe(stub.cancellable()).await;
        Self {
            observer,
            stub,
            phantom: PhantomData,
        }
    }

    pub async fn on_next(&mut self, item: Item) {
        self.observer.on_next(item).await;
    }

    pub async fn on_error(&mut self, error: Error) {
        self.observer.on_error(error).await;
    }

    pub async fn on_completed(&mut self) {
        self.observer.on_completed().await;
    }

    pub fn is_cancelled(&self) -> bool {
        self.stub.is_cancelled()
    }
}
