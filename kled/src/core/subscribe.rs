use crate::core;
use crate::util;

pub trait ObservableSubcribe<Downstream>: util::ObservableSealed {
    fn subscribe(self, _: Downstream);
}

impl<Observer, Observable> ObservableSubcribe<Observer> for Observable
where
    Observer: core::Observer<Observable::Cancellable, Observable::Item, Observable::Error>
        + Send
        + 'static,
    Observable: core::Observable + Send + 'static,
{
    fn subscribe(self, observer: Observer) {
        self.actual_subscribe(observer);
    }
}

pub trait FlowSubcribe<Downstream>: util::FlowSealed {
    fn subscribe(self, _: Downstream);
}

impl<Subscriber, Flow> FlowSubcribe<Subscriber> for Flow
where
    Subscriber: core::Subscriber<Flow::Subscription, Flow::Item, Flow::Error> + Send + 'static,
    Flow: core::Flow + Send + 'static,
{
    fn subscribe(self, subscriber: Subscriber) {
        self.actual_subscribe(subscriber);
    }
}

pub trait ObservableSubsribeNext<NextFn>: util::ObservableSealed {
    type Cancellable: core::Cancellable;

    fn subscribe_next(self, _: NextFn) -> Self::Cancellable;
}

pub trait FlowSubsribeNext<NextFn>: util::FlowSealed {
    type Subscription: core::Subscription;

    fn subscribe_next(self, _: NextFn) -> Self::Subscription;
}

pub trait ObservableSubsribeAll<NextFn, ErrorFn, CompletedFn>: util::ObservableSealed {
    type Cancellable: core::Cancellable;

    fn subscribe_all(self, _: NextFn, _: ErrorFn, _: CompletedFn) -> Self::Cancellable;
}

pub trait FlowSubsribeAll<NextFn, ErrorFn, CompletedFn>: util::FlowSealed {
    type Subscription: core::Subscription;

    fn subscribe_all(self, _: NextFn, _: ErrorFn, _: CompletedFn) -> Self::Subscription;
}
