use crate::core;

pub trait LocalObservable<'o>: core::Observable {
    type Cancellable: core::Cancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + 'o;
}
