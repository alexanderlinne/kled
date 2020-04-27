use crate::core;

pub trait SharedObservable: core::Observable {
    type Observation;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::Observer<Self::Observation, Self::Item, Self::Error> + Send + Sync + 'static;

    fn into_shared(self) -> core::Shared<Self>
    where
        Self: Sized,
    {
        core::Shared {
            actual_observable: self,
        }
    }
}
