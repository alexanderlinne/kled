use crate::core;

pub struct Shared<Observable> {
    pub(crate) actual_observable: Observable,
}

impl<T> core::Observable for Shared<T>
where
    T: core::Observable,
{
    type Item = T::Item;
    type Error = T::Error;
}

impl<T> core::SharedObservable for Shared<T>
where
    T: core::SharedObservable,
{
    type Observation = T::Observation;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer:
            core::observer::Observer<T::Observation, T::Item, T::Error> + Send + Sync + 'static,
    {
        self.actual_observable.actual_subscribe(observer)
    }
}
