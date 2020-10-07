use crate::cancellable::local::*;
use crate::core;
use std::marker::PhantomData;

pub struct FromObserver<Observer, Item, Error> {
    observer: Observer,
    cancellable: BoolCancellable,
    phantom: PhantomData<(Item, Error)>,
}

impl<'o, Observer, Item, Error> FromObserver<Observer, Item, Error>
where
    Observer: core::Observer<BoolCancellable, Item, Error> + 'o,
{
    pub fn new(mut observer: Observer) -> Self {
        let cancellable = BoolCancellable::default();
        observer.on_subscribe(cancellable.clone());
        Self {
            observer,
            cancellable,
            phantom: PhantomData,
        }
    }
}

impl<'o, Observer, Item, Error> core::ObservableEmitter<Item, Error>
    for FromObserver<Observer, Item, Error>
where
    Observer: core::Observer<BoolCancellable, Item, Error> + 'o,
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

    fn is_cancelled(&self) -> bool {
        use self::core::Cancellable;
        self.cancellable.is_cancelled()
    }
}

impl<'o, Observer, Item, Error> core::IntoObservableEmitter<'o, Item, Error> for Observer
where
    Observer: core::Observer<BoolCancellable, Item, Error> + 'o,
{
    type Emitter = FromObserver<Observer, Item, Error>;

    fn into_emitter(self) -> Self::Emitter {
        FromObserver::new(self)
    }
}
