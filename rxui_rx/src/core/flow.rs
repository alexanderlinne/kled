use crate::core;

pub trait LocalFlow<'o>: Flow {
    type Subscription: core::Subscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + 'o;
}

pub trait SharedFlow: Flow {
    type Subscription: core::Subscription;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + Send + 'static;
}

pub trait Flow {
    type Item;
    type Error;
}
