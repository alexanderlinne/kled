use crate::core;
use crate::core::private::Sealed;

pub trait Subcribe<'o, ObserverOrSubscriber>: Sealed {
    fn subscribe(self, _: ObserverOrSubscriber);
}

impl<'o, Observable> Sealed for Observable where Observable: core::LocalObservable<'o> + 'o {}

impl<'o, Observer, Observable> Subcribe<'o, Observer> for Observable
where
    Observer: core::Observer<Observable::Cancellable, Observable::Item, Observable::Error> + 'o,
    Observable: core::LocalObservable<'o> + 'o,
{
    fn subscribe(self, observer: Observer) {
        self.actual_subscribe(observer);
    }
}

impl<Observable> Sealed for core::Shared<Observable> where
    Observable: core::SharedObservable + Send + 'static
{
}

impl<'o, Observer, Observable> Subcribe<'o, Observer> for core::Shared<Observable>
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
