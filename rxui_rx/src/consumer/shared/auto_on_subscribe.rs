use crate::core;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct AutoOnSubscribe<Observer, Item, Error> {
    observer: Observer,
    observed: Arc<AtomicBool>,
    phantom: PhantomData<(Item, Error)>,
}

impl<Observer, Item, Error> AutoOnSubscribe<Observer, Item, Error>
where
    Observer: core::Observer<core::SharedCancellable, Item, Error>,
{
    pub fn new(mut observer: Observer) -> Self {
        let observed = Arc::new(AtomicBool::new(true));
        observer.on_subscribe(core::SharedCancellable::new(observed.clone()));
        Self {
            observer,
            observed,
            phantom: PhantomData,
        }
    }
}

impl<ObserverType, Item, Error> core::Consumer<Item, Error>
    for AutoOnSubscribe<ObserverType, Item, Error>
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

impl<ObserverType, Item, Error> core::CancellableConsumer<Item, Error>
    for AutoOnSubscribe<ObserverType, Item, Error>
where
    ObserverType: core::Observer<core::SharedCancellable, Item, Error>,
{
    fn is_cancelled(&self) -> bool {
        !self.observed.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use crate::observer;
    use crate::prelude::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn unsubscribe_shared() {
        let vec = vec![0, 1, 2, 3];
        let sum = Arc::new(Mutex::new(0));
        let sum_move = sum.clone();
        vec.into_observable()
            .into_shared()
            .subscribe(observer::from_fn(
                |sub: SharedCancellable| {
                    sub.cancel();
                },
                move |v| *sum_move.lock().unwrap() += v,
                |_| {},
                || {},
            ));
        assert_eq!(*sum.lock().unwrap(), 0);
    }
}
