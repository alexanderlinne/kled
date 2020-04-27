use crate::core;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub trait SharedObservable: core::Observable {
    type Subscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::Observer<Self::Subscription, Self::Item, Self::Error> + Send + Sync + 'static;

    fn into_shared(self) -> Shared<Self>
    where
        Self: Sized,
    {
        Shared {
            actual_observable: self,
        }
    }
}

pub struct Shared<Observable> {
    pub(crate) actual_observable: Observable,
}

impl<T> core::Observable for Shared<T>
where
    T: core::Observable,
{
    type Item = T::Item;
    type Error = T::Error;
}

impl<T> SharedObservable for Shared<T>
where
    T: SharedObservable,
{
    type Subscription = T::Subscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::observer::Observer<T::Subscription, T::Item, T::Error> + Send + Sync + 'static,
    {
        self.actual_observable.actual_subscribe(observer)
    }
}

pub struct SharedSubscription {
    subscribed: Arc<AtomicBool>,
}

impl SharedSubscription {
    pub(crate) fn new(subscribed: Arc<AtomicBool>) -> Self {
        Self { subscribed }
    }
}

impl core::Subscription for SharedSubscription {
    fn unsubscribe(self) {
        self.subscribed.store(false, Ordering::Release);
    }
}
