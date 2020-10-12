use crate::core;
use crate::flow;
use crate::flow::operators::*;

pub trait Flow<Subscription, Item, Error> {
    fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static;

    fn map<ItemOut, UnaryOp>(
        self,
        unary_op: UnaryOp,
    ) -> Map<Self, Subscription, Item, Error, ItemOut, UnaryOp>
    where
        Self: Sized,
        UnaryOp: FnMut(Item) -> ItemOut,
    {
        Map::new(self, unary_op)
    }

    fn observe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> ObserveOn<Self, Subscription, Item, Error, Scheduler>
    where
        Self: Sized,
        Item: Send,
        Error: Send,
        Scheduler: core::Scheduler + Send + 'static,
    {
        ObserveOn::new(self, scheduler)
    }

    fn on_backpressure_buffer(
        self,
        buffer_strategy: flow::BufferStrategy,
    ) -> OnBackpressureBuffer<Self, Subscription, Item, Error>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
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
        Item: Send + 'static,
        Error: Send + 'static,
    {
        OnBackpressureBuffer::new(self, buffer_strategy, capacity)
    }

    fn on_backpressure_drop(self) -> OnBackpressureDrop<Self, Subscription, Item, Error>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
    {
        OnBackpressureDrop::new(self)
    }

    fn on_backpressure_error(self) -> OnBackpressureError<Self, Subscription, Item, Error>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
    {
        OnBackpressureError::new(self)
    }

    fn on_backpressure_latest(self) -> OnBackpressureLatest<Self, Subscription, Item, Error>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
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
        ItemOut: Clone,
        BinaryOp: FnMut(ItemOut, Item) -> ItemOut,
    {
        Scan::new(self, initial_value, binary_op)
    }

    fn subscribe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> SubscribeOn<Self, Subscription, Item, Error, Scheduler>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
        Scheduler: core::Scheduler + Send + 'static,
    {
        SubscribeOn::new(self, scheduler)
    }
}
