use crate::core;

pub trait ObserverSubscribe<'o, Observer> {
    fn subscribe(self, observer: Observer);
}

impl<'o, Observer, Observable> ObserverSubscribe<'o, Observer> for Observable
where
    Observer: core::Observer<Observable::Observation, Observable::Item, Observable::Error> + 'o,
    Observable: core::LocalObservable<'o> + 'o,
{
    fn subscribe(self, observer: Observer) {
        self.actual_subscribe(observer);
    }
}

impl<'o, Observer, Observable> ObserverSubscribe<'o, Observer> for core::Shared<Observable>
where
    Observer: core::Observer<Observable::Observation, Observable::Item, Observable::Error>
        + Send
        + Sync
        + 'static,
    Observable: core::SharedObservable + Send + Sync + 'static,
{
    fn subscribe(self, observer: Observer) {
        self.actual_observable.actual_subscribe(observer);
    }
}
