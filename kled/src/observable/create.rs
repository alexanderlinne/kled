use crate::cancellable::*;
use crate::core;
use crate::observable;
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

impl<F, Item, Error> core::Observable<BoolCancellable, Item, Error>
    for ObservableCreate<F, Item, Error>
where
    F: FnOnce(observable::BoxEmitter<Item, Error>),
    Item: Send + 'static,
    Error: Send + 'static,
{
    fn subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<BoolCancellable, Item, Error> + Send + 'static,
    {
        let emitter = observable::BoxEmitter::from(observer);
        (self.emitter_consumer)(emitter);
    }
}

pub fn create<F, Item, Error>(emitter_consumer: F) -> ObservableCreate<F, Item, Error>
where
    F: FnOnce(observable::BoxEmitter<Item, Error>),
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
