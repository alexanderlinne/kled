use crate::core;

pub trait ObservableSubcribe<'o, Observer>: core::util::Sealed {
    fn subscribe(self, _: Observer);
}

impl<'o, Observer, Observable> ObservableSubcribe<'o, Observer> for Observable
where
    Observer: core::Observer<Observable::Cancellable, Observable::Item, Observable::Error> + 'o,
    Observable: core::LocalObservable<'o> + 'o,
{
    fn subscribe(self, observer: Observer) {
        self.actual_subscribe(observer);
    }
}

impl<'o, Observer, Observable> ObservableSubcribe<'o, Observer> for core::Shared<Observable>
where
    Observer: core::Observer<Observable::Cancellable, Observable::Item, Observable::Error>
        + Send
        + 'static,
    Observable: core::SharedObservable + Send + 'static,
{
    fn subscribe(self, observer: Observer) {
        self.actual_observable.actual_subscribe(observer);
    }
}
