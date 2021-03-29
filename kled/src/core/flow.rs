use crate::{core, flow, subscriber};
use crate::flow::operators::*;
use crate::subscription::LazySubscription;
use async_trait::async_trait;

/// A backpressured source of `Item`s to which a [`Subscriber`] may subscribe.
///
/// `Flow` is the base trait which any flow type must implement. It defines the
/// type of `Item`s and `Error`s it may emit and the [`Subscription`] type the
/// flow passes to the subscriber.
///
/// [`Subscriber`]: trait.Subscriber.html
/// [`Subscription`]: trait.Subscription.html
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

    async fn subscribe_next<NextFn>(self, next_fn: NextFn) -> LazySubscription<Subscription>
    where
        Self: Sized,
        NextFn: FnMut(Item) + Send + 'static,
    {
        let subscriber = subscriber::LambdaSubscriber::new(
            next_fn,
            |_| {
                panic! {}
            },
            || {},
        );
        let subscription = subscriber.subscription();
        self.subscribe(subscriber).await;
        subscription
    }

    async fn subscribe_all<NextFn, ErrorFn, CompletedFn>(
        self,
        next_fn: NextFn,
        error_fn: ErrorFn,
        complete_fn: CompletedFn,
    ) -> LazySubscription<Subscription>
    where
        Self: Sized,
        NextFn: FnMut(Item) + Send + 'static,
        ErrorFn: FnMut(flow::Error<Error>) + Send + 'static,
        CompletedFn: FnMut() + Send + 'static,
    {
        let subscriber = subscriber::LambdaSubscriber::new(next_fn, error_fn, complete_fn);
        let subscription = subscriber.subscription();
        self.subscribe(subscriber).await;
        subscription
    }

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
