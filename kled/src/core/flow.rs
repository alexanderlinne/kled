use crate::core;
use crate::flow;
use crate::flow::operators::*;
use async_trait::async_trait;

/// A backpressured source of `Item`s to which an [`Subscriber`] may subscribe.
///
/// [`Flow`] is the base trait which any flow type must implement. It defines the
/// type of `Item`s and `Error`s it may emit and the `Subscription` type the flow passes
/// to the [`Subscriber`] via [`Subscriber::on_subscribe`].
///
/// The core operators for [`Flow`]s are provided via the [`FlowExt`] trait.
///
/// [`Subscriber`]: trait.Subscriber.html
/// [`FlowExt`]: trait.FlowExt.html
/// [`Subscriber::on_subscribe`]: trait.Subscriber.html#tymethod.on_subscribe
#[async_trait]
pub trait Flow<Subscription, Item, Error>
where
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static;
}

pub trait IntoFlow<Subscription, Item, Error>
where
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Flow: core::Flow<Subscription, Item, Error>;

    fn into_flow(self) -> Self::Flow;
}

impl<T: ?Sized, Subscription, Item, Error> FlowExt<Subscription, Item, Error> for T
where
    T: Flow<Subscription, Item, Error>,
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
}


/// An extension trait for [`Flow`] that provides core operators.
///
/// [`Flow`]: trait.Flow.html
pub trait FlowExt<Subscription, Item, Error>: Flow<Subscription, Item, Error>
where
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    fn dematerialize(
        self,
    ) -> Dematerialize<Self, Subscription, Item, Error>
    where
        Self: Sized,
    {
        Dematerialize::new(self)
    }

    fn map<ItemOut, UnaryOp>(
        self,
        unary_op: UnaryOp,
    ) -> Map<Self, Subscription, Item, Error, ItemOut, UnaryOp>
    where
        Self: Sized,
        UnaryOp: FnMut(Item) -> ItemOut + Send + 'static,
    {
        Map::new(self, unary_op)
    }

    fn materialize(
        self,
    ) -> Materialize<Self, Subscription, Item, Error>
    where
        Self: Sized,
    {
        Materialize::new(self)
    }

    fn observe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> ObserveOn<Self, Subscription, Item, Error, Scheduler>
    where
        Self: Sized,
        Scheduler: core::Scheduler + Send + 'static,
    {
        Dematerialize::new(ObserveOnRaw::new(Materialize::new(self), scheduler))
    }

    fn on_backpressure_buffer(
        self,
        buffer_strategy: flow::BufferStrategy,
    ) -> OnBackpressureBuffer<Self, Subscription, Item, Error>
    where
        Self: Sized,
    {
        OnBackpressureBuffer::new(self, buffer_strategy, flow::default_buffer_capacity())
    }

    fn on_backpressure_buffer_with_capacity(
        self,
        buffer_strategy: flow::BufferStrategy,
        capacity: usize,
    ) -> OnBackpressureBuffer<Self, Subscription, Item, Error>
    where
        Self: Sized,
    {
        OnBackpressureBuffer::new(self, buffer_strategy, capacity)
    }

    fn on_backpressure_drop(self) -> OnBackpressureDrop<Self, Subscription, Item, Error>
    where
        Self: Sized,
    {
        OnBackpressureDrop::new(self)
    }

    fn on_backpressure_error(self) -> OnBackpressureError<Self, Subscription, Item, Error>
    where
        Self: Sized,
    {
        OnBackpressureError::new(self)
    }

    fn on_backpressure_latest(self) -> OnBackpressureLatest<Self, Subscription, Item, Error>
    where
        Self: Sized,
    {
        OnBackpressureLatest::new(self)
    }

    fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> Scan<Self, Subscription, Item, Error, ItemOut, BinaryOp>
    where
        Self: Sized,
        ItemOut: Clone + Send + 'static,
        BinaryOp: FnMut(ItemOut, Item) -> ItemOut + Send + 'static,
    {
        Scan::new(self, initial_value, binary_op)
    }

    fn subscribe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> SubscribeOn<Self, Subscription, Item, Error, Scheduler>
    where
        Self: Sized,
        Scheduler: core::Scheduler + Send + 'static,
    {
        SubscribeOn::new(self, scheduler)
    }
}
