use crate::core;

pub trait LocalObservable<'o>: core::Observable {
    type Subscription;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Subscription, Self::Item, Self::Error> + 'o;
}
