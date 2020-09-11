use crate::core;
use crate::emitter;
use std::marker::PhantomData;

#[derive(Clone)]
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

impl<F, Item, Error> core::Observable for ObservableCreate<F, Item, Error> {
    type Item = Item;
    type Error = Error;
}

impl<'o, F, Item, Error> core::LocalObservable<'o> for ObservableCreate<F, Item, Error>
where
    F: FnOnce(Box<dyn core::CancellableEmitter<Item, Error> + 'o>),
    Item: 'o,
    Error: 'o,
{
    type Cancellable = core::LocalCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + 'o,
    {
        let emitter = emitter::local::FromObserver::new(observer);
        (self.emitter_consumer)(Box::new(emitter));
    }
}

impl<F, Item, Error> core::SharedObservable for ObservableCreate<F, Item, Error>
where
    F: FnOnce(Box<dyn core::CancellableEmitter<Item, Error> + Send>),
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Cancellable = core::SharedCancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        let emitter = emitter::shared::FromObserver::new(observer);
        (self.emitter_consumer)(Box::new(emitter));
    }
}

pub fn create<F, Item, Error>(observer_consumer: F) -> ObservableCreate<F, Item, Error> {
    ObservableCreate::new(observer_consumer)
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::util::shared::*;

    #[test]
    fn create() {
        let test_observer = TestObserver::default();
        observable::create(|mut emitter: Box<dyn CancellableEmitter<i32, ()>>| {
            emitter.on_next(0);
            emitter.on_completed();
        })
        .subscribe(test_observer.clone());
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0]);
    }
}
