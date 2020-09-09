use crate::core;
use crate::operators;

/// A non-backpressured source of [`Item`]s to which an [`Observer`] may subscribe.
///
/// [`Observable`] is the base trait which any observable type must implement. It defines the
/// type of [`Item`]s and [`Error`]s it may emit. This trait is specialized by
/// the [`LocalObservable`] and [`SharedObservable`] traits which define the type of
/// cancellable passed to [`Observer::on_subscribe`] and the function to subscribe an observer.
/// The [`SharedObservable`] additionally requires those to be thread-safe.
///
/// [`Observer`]: trait.Observer.html
/// [`Observer::on_subscribe`]: trait.Observer.html#tymethod.on_subscribe
/// [`Item`]: trait.Observable.html#associatedtype.Item
/// [`Error`]: trait.Observable.html#associatedtype.Error
/// [`LocalObservable`]: trait.LocalObservable.html
/// [`SharedObservable`]: trait.SharedObservable.html
pub trait Observable {
    /// The type of items emitted by this observable.
    type Item;

    /// The type of error which may be emitted by this observable.
    type Error;

    fn observe_on<Scheduler>(
        self,
        scheduler: &Scheduler,
    ) -> core::Shared<operators::ObserveOn<Self, Scheduler::Worker>>
    where
        Self: Sized,
        Scheduler: core::Scheduler,
    {
        core::Shared::new(operators::ObserveOn::new(self, scheduler.create_worker()))
    }
}
