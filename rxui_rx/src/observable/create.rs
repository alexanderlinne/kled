use crate::cancellable;
use crate::core;
use crate::core::{IntoObservableEmitter, IntoSharedObservableEmitter};
use std::marker::PhantomData;

#[derive(Clone)]
#[doc(hidden)]
pub struct ObservableCreate<F, Item, Error> {
    emitter_consumer: F,
    phantom: PhantomData<(Item, Error)>,
}

impl<F, Item, Error> ObservableCreate<F, Item, Error> {
    pub fn new(emitter_consumer: F) -> Self {
        ObservableCreate {
            emitter_consumer,
            phantom: PhantomData,
        }
    }
}

impl<'o, F, Item, Error> core::LocalObservable<'o> for ObservableCreate<F, Item, Error>
where
    F: FnOnce(Box<dyn core::ObservableEmitter<Item, Error> + 'o>),
    Item: 'o,
    Error: 'o,
{
    type Cancellable = cancellable::local::BoolCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + 'o,
    {
        let emitter = observer.into_emitter();
        (self.emitter_consumer)(Box::new(emitter));
    }
}

impl<F, Item, Error> core::SharedObservable for ObservableCreate<F, Item, Error>
where
    F: FnOnce(Box<dyn core::ObservableEmitter<Item, Error> + Send>),
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Cancellable = cancellable::shared::BoolCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        let emitter = observer.into_shared_emitter();
        (self.emitter_consumer)(Box::new(emitter));
    }
}

impl<F, Item, Error> core::Observable for ObservableCreate<F, Item, Error> {
    type Item = Item;
    type Error = Error;
}

pub fn create<F, Item, Error>(emitter_consumer: F) -> ObservableCreate<F, Item, Error> {
    ObservableCreate::new(emitter_consumer)
}

#[cfg(test)]
mod tests {
    use crate::observer::local::*;
    use crate::prelude::*;

    #[test]
    fn create() {
        let test_observer = TestObserver::default();
        observable::create(|mut emitter: Box<dyn ObservableEmitter<i32, ()>>| {
            emitter.on_next(0);
            emitter.on_completed();
        })
        .subscribe(test_observer.clone());
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0]);
    }
}
