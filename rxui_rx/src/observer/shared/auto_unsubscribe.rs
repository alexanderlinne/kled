use crate::core;
use crate::core::AutoUnsubscribeObserver;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct AutoUnscubscribe<ObserverType, Subscription, Item, Error> {
    observer: ObserverType,
    subscribed: Arc<AtomicBool>,
    phantom: PhantomData<(Subscription, Item, Error)>,
}

impl<ObserverType, Subscription, Item, Error>
    AutoUnscubscribe<ObserverType, Subscription, Item, Error>
where
    ObserverType: core::Observer<Subscription, Item, Error>,
{
    pub fn new(observer: ObserverType) -> Self {
        Self {
            observer,
            subscribed: Arc::new(AtomicBool::new(true)),
            phantom: PhantomData,
        }
    }

    pub fn create_subscription(&self) -> core::SharedSubscription {
        core::SharedSubscription::new(self.subscribed.clone())
    }
}

impl<ObserverType, Subscription, Item, Error>
    core::AutoUnsubscribeObserver<Subscription, Item, Error>
    for AutoUnscubscribe<ObserverType, Subscription, Item, Error>
where
    ObserverType: core::Observer<Subscription, Item, Error>,
{
    fn is_unsubscribed(&self) -> bool {
        !self.subscribed.load(Ordering::Acquire)
    }
}

impl<ObserverType, Subscription, Item, Error> core::Observer<Subscription, Item, Error>
    for AutoUnscubscribe<ObserverType, Subscription, Item, Error>
where
    ObserverType: core::Observer<Subscription, Item, Error>,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        if !self.is_unsubscribed() {
            self.observer.on_subscribe(subscription);
        }
    }

    fn on_next(&mut self, item: Item) {
        if !self.is_unsubscribed() {
            self.observer.on_next(item);
        }
    }

    fn on_error(&mut self, error: Error) {
        if !self.is_unsubscribed() {
            self.observer.on_error(error);
        }
    }

    fn on_completed(&mut self) {
        if !self.is_unsubscribed() {
            self.observer.on_completed();
        }
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
                |sub: SharedSubscription| {
                    sub.unsubscribe();
                },
                move |v| *sum_move.lock().unwrap() += v,
                |_| {},
                || {},
            ));
        assert_eq!(*sum.lock().unwrap(), 0);
    }
}
