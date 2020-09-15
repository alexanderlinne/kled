use crate::core;
use crate::marker;
use crate::util;

pub trait Subcribe<'o, Downstream>: util::Sealed {
    fn subscribe(self, _: Downstream);
}

impl<'o, Observer, Observable> Subcribe<'o, Observer> for Observable
where
    Observer: core::Observer<Observable::Cancellable, Observable::Item, Observable::Error> + 'o,
    Observable: core::LocalObservable<'o> + 'o,
{
    fn subscribe(self, observer: Observer) {
        self.actual_subscribe(observer);
    }
}

impl<'o, Observer, Observable> Subcribe<'o, Observer> for marker::Shared<Observable>
where
    Observer: core::Observer<Observable::Cancellable, Observable::Item, Observable::Error>
        + Send
        + 'static,
    Observable: core::SharedObservable + Send + 'static,
{
    fn subscribe(self, observer: Observer) {
        self.actual.actual_subscribe(observer);
    }
}

impl<'o, Subscriber, Flow> Subcribe<'o, Subscriber> for marker::Flow<Flow>
where
    Subscriber: core::Subscriber<Flow::Subscription, Flow::Item, Flow::Error> + 'o,
    Flow: core::LocalFlow<'o> + 'o,
{
    fn subscribe(self, subscriber: Subscriber) {
        self.actual.actual_subscribe(subscriber);
    }
}

impl<'o, Subscriber, Flow> Subcribe<'o, Subscriber> for marker::Shared<marker::Flow<Flow>>
where
    Subscriber: core::Subscriber<Flow::Subscription, Flow::Item, Flow::Error> + Send + 'static,
    Flow: core::SharedFlow + Send + 'static,
{
    fn subscribe(self, subscriber: Subscriber) {
        self.actual.actual.actual_subscribe(subscriber);
    }
}

pub trait ObservableSubsribeNext<'o, NextFn>: util::Sealed {
    type Cancellable: core::Cancellable;

    fn subscribe_next(self, _: NextFn) -> Self::Cancellable;
}

pub trait ObservableSubsribeAll<'o, NextFn, ErrorFn, CompletedFn>: util::Sealed {
    type Cancellable: core::Cancellable;

    fn subscribe_all(self, _: NextFn, _: ErrorFn, _: CompletedFn) -> Self::Cancellable;
}
