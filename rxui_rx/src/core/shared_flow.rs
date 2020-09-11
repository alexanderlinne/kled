use crate::core;

pub trait SharedFlow: core::Flow {
    type Subscription: core::Subscription;

    fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + Send + 'static;

    fn into_shared(self) -> core::Shared<Self>
    where
        Self: Sized,
    {
        core::Shared::new(self)
    }
}
