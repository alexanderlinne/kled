use crate::core;

pub trait SharedObservable: core::Observable {
    type Subscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Subscription, Self::Item, Self::Error> + Send + 'static;

    fn into_shared(self) -> core::Shared<Self>
    where
        Self: Sized,
    {
        core::Shared::new(self)
    }
}
