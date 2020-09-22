use crate::core;
use crate::marker;
use crate::operators;

pub trait LocalObservable<'o>: Observable {
    type Cancellable: core::Cancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + 'o;

    fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> operators::ObservableScan<Self, ItemOut, BinaryOp>
    where
        Self: Sized,
        ItemOut: Clone,
        BinaryOp: FnMut(ItemOut, Self::Item) -> ItemOut,
    {
        operators::ObservableScan::new(self, initial_value, binary_op)
    }
}

pub trait SharedObservable: Observable {
    type Cancellable: core::Cancellable + Send + Sync + 'static;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static;

    fn into_shared(self) -> marker::Shared<Self>
    where
        Self: Sized,
    {
        marker::Shared::new(self)
    }
}

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
    /// The type of items emitted by this observable
    type Item;

    /// The type of error which may be emitted by this observable
    type Error;

    fn observe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> marker::Shared<operators::ObservableObserveOn<Self, Scheduler>>
    where
        Self: SharedObservable + Sized,
        Self::Cancellable: Send,
        Self::Item: Send,
        Self::Error: Send,
        Scheduler: core::Scheduler + Send,
    {
        marker::Shared::new(operators::ObservableObserveOn::new(self, scheduler))
    }

    fn subscribe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> marker::Shared<operators::ObservableSubscribeOn<Self, Scheduler>>
    where
        Self: Sized,
        Scheduler: core::Scheduler + Send,
    {
        marker::Shared::new(operators::ObservableSubscribeOn::new(self, scheduler))
    }
}
