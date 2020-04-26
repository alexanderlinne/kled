use crate::core;

pub trait Observable {
    type Item;
    type Error;
    type SubscriptionType: Subscription + Send + Sync + 'static;

    fn subscribe<ObserverType>(self, observer: ObserverType) -> Self::SubscriptionType
    where
        ObserverType: core::Observer<Self::Item, Self::Error> + Send + Sync + 'static;
}

pub trait Subscription {
    fn unsubscribe(&mut self);
}
