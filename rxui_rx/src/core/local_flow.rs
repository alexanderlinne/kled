use crate::core;

pub trait LocalFlow<'o>: core::Flow {
    type Subscription: core::Subscription;

    fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + 'o;
}
