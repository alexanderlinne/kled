use crate::core;
use crate::observable::operators::*;

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
pub trait Observable<Cancellable, Item, Error>
where
    Cancellable: core::Cancellable + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    fn subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Cancellable, Item, Error> + Send + 'static;
}

impl<T: ?Sized, Cancellable, Item, Error> ObservableExt<Cancellable, Item, Error> for T
where
    T: Observable<Cancellable, Item, Error>,
    Cancellable: core::Cancellable + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
}

pub trait ObservableExt<Cancellable, Item, Error>: Observable<Cancellable, Item, Error>
where
    Cancellable: core::Cancellable + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    fn map<ItemOut, UnaryOp>(
        self,
        unary_op: UnaryOp,
    ) -> Map<Self, Cancellable, Item, Error, ItemOut, UnaryOp>
    where
        Self: Sized,
        UnaryOp: FnMut(Item) -> ItemOut + Send + 'static,
    {
        Map::new(self, unary_op)
    }

    fn observe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> ObserveOn<Self, Cancellable, Item, Error, Scheduler>
    where
        Self: Sized,
        Scheduler: core::Scheduler + Send + 'static,
    {
        ObserveOn::new(self, scheduler)
    }

    fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> Scan<Self, Cancellable, Item, Error, ItemOut, BinaryOp>
    where
        Self: Sized,
        ItemOut: Clone + Send + 'static,
        BinaryOp: FnMut(ItemOut, Item) -> ItemOut,
    {
        Scan::new(self, initial_value, binary_op)
    }

    fn subscribe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> SubscribeOn<Self, Cancellable, Item, Error, Scheduler>
    where
        Self: Sized,
        Scheduler: core::Scheduler + Send + 'static,
    {
        SubscribeOn::new(self, scheduler)
    }
}
