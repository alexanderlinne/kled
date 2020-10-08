use crate::cancellable;
use crate::core;
use crate::core::IntoObservableEmitter;
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

impl<F, Item, Error> core::Observable for ObservableCreate<F, Item, Error>
where
    F: FnOnce(Box<dyn core::ObservableEmitter<Item, Error> + Send>),
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Item = Item;
    type Error = Error;
    type Cancellable = cancellable::BoolCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        let emitter = observer.into_emitter();
        (self.emitter_consumer)(Box::new(emitter));
    }
}

pub fn create<F, Item, Error>(emitter_consumer: F) -> ObservableCreate<F, Item, Error>
where
    F: FnOnce(Box<dyn core::ObservableEmitter<Item, Error> + Send>),
    Item: Send + 'static,
    Error: Send + 'static,
{
    ObservableCreate::new(emitter_consumer)
}

#[cfg(test)]
mod tests {
    use crate::observer::*;
    use crate::prelude::*;

    #[test]
    fn create() {
        let test_observer = TestObserver::default();
        observable::create(|mut emitter| {
            if false {
                emitter.on_error(())
            };
            emitter.on_next(0);
            emitter.on_completed();
        })
        .subscribe(test_observer.clone());
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0]);
    }
}