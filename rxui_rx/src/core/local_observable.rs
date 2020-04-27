use crate::core;

pub trait LocalObservable<'o>: core::Observable {
    type Observation;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Observation, Self::Item, Self::Error> + 'o;
}
