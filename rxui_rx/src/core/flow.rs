use crate::core;

pub trait Flow {
    type Item;
    type Error;

    fn subscribe<SubscriberType>(self, subscriber: SubscriberType)
    where
        SubscriberType: core::Subscriber<Self::Item, Self::Error> + Send + Sync + 'static;
}

pub trait Subscription {
    fn unsubscribe(&mut self);

    fn request(&mut self, count: usize);
}

pub trait IntoFlow {
    type FlowType: Flow;

    fn into_flow(self) -> Self::FlowType;
}
