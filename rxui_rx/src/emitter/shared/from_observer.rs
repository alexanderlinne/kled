use crate::core;
use std::marker::PhantomData;

pub struct FromObserver<Observer, Item, Error> {
    observer: Observer,
    cancellable: core::SharedCancellable,
    phantom: PhantomData<(Item, Error)>,
}

impl<Observer, Item, Error> FromObserver<Observer, Item, Error>
where
    Observer: core::Observer<core::SharedCancellable, Item, Error>,
{
    pub fn new(mut observer: Observer) -> Self {
        let cancellable = core::SharedCancellable::default();
        observer.on_subscribe(cancellable.clone());
        Self {
            observer,
            cancellable,
            phantom: PhantomData,
        }
    }
}

impl<ObserverType, Item, Error> core::Emitter<Item, Error>
    for FromObserver<ObserverType, Item, Error>
where
    ObserverType: core::Observer<core::SharedCancellable, Item, Error>,
{
    fn on_next(&mut self, item: Item) {
        self.observer.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        self.observer.on_error(error);
    }

    fn on_completed(&mut self) {
        self.observer.on_completed();
    }
}

impl<ObserverType, Item, Error> core::CancellableEmitter<Item, Error>
    for FromObserver<ObserverType, Item, Error>
where
    ObserverType: core::Observer<core::SharedCancellable, Item, Error>,
{
    fn is_cancelled(&self) -> bool {
        use self::core::Cancellable;
        self.cancellable.is_cancelled()
    }
}
