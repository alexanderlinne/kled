use crate::core;
use crate::core::AutoUnsubscribeObserver;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct AutoUnscubscribe<ObserverType, Item, Error> {
    observer: ObserverType,
    subscribed: Arc<AtomicBool>,
    phantom: PhantomData<(Item, Error)>,
}

impl<ObserverType, Item, Error> AutoUnscubscribe<ObserverType, Item, Error>
where
    ObserverType: core::Observer<Item, Error> + Send + Sync + 'static,
{
    pub fn new(observer: ObserverType) -> Self {
        Self {
            observer,
            subscribed: Arc::new(AtomicBool::new(true)),
            phantom: PhantomData,
        }
    }

    pub fn create_subscription(&self) -> AutoUnscubscribeSubscription {
        AutoUnscubscribeSubscription::new(self.subscribed.clone())
    }
}

impl<ObserverType, Item, Error> core::AutoUnsubscribeObserver<Item, Error>
    for AutoUnscubscribe<ObserverType, Item, Error>
where
    ObserverType: core::Observer<Item, Error> + Send + Sync + 'static,
{
    fn is_unsubscribed(&self) -> bool {
        !self.subscribed.load(Ordering::Acquire)
    }
}

impl<ObserverType, Item, Error> core::Observer<Item, Error>
    for AutoUnscubscribe<ObserverType, Item, Error>
where
    ObserverType: core::Observer<Item, Error> + Send + Sync + 'static,
{
    fn on_subscribe(&mut self, subscription: Box<dyn core::observable::Subscription>) {
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

pub struct AutoUnscubscribeSubscription {
    subscribed: Arc<AtomicBool>,
}

impl AutoUnscubscribeSubscription {
    pub(crate) fn new(subscribed: Arc<AtomicBool>) -> Self {
        Self { subscribed }
    }
}

impl core::observable::Subscription for AutoUnscubscribeSubscription {
    fn unsubscribe(&mut self) {
        self.subscribed.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use crate::observer;
    use crate::prelude::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn unsubscribe() {
        let vec = vec![0, 1, 2, 3];
        let sum = Arc::new(Mutex::new(0));
        let sum_move = sum.clone();
        vec.into_observable().subscribe(observer::from_fn(
            |mut sub| {
                sub.unsubscribe();
            },
            move |v| (*sum_move.lock().unwrap()) += v,
            |_| {},
            || {},
        ));
        assert_eq!((*sum.lock().unwrap()), 0);
    }
}
