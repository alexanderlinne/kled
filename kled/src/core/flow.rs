use crate::core;
use crate::flow;
use crate::operators::*;

pub trait Flow<Subscription, Item, Error> {
    fn subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Subscription, Item, Error> + Send + 'static;

    fn map<ItemOut, UnaryOp>(
        self,
        unary_op: UnaryOp,
    ) -> FlowMap<Self, Subscription, Item, Error, ItemOut, UnaryOp>
    where
        Self: Sized,
        UnaryOp: FnMut(Item) -> ItemOut,
    {
        FlowMap::new(self, unary_op)
    }

    fn observe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> FlowObserveOn<Self, Subscription, Item, Error, Scheduler>
    where
        Self: Sized,
        Item: Send,
        Error: Send,
        Scheduler: core::Scheduler + Send + 'static,
    {
        FlowObserveOn::new(self, scheduler)
    }

    fn on_backpressure_buffer(
        self,
        buffer_strategy: flow::BufferStrategy,
    ) -> FlowOnBackpressureBuffer<Self, Subscription, Item, Error>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
    {
        FlowOnBackpressureBuffer::new(self, buffer_strategy, flow::default_buffer_capacity())
    }

    fn on_backpressure_buffer_with_capacity(
        self,
        buffer_strategy: flow::BufferStrategy,
        capacity: usize,
    ) -> FlowOnBackpressureBuffer<Self, Subscription, Item, Error>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
    {
        FlowOnBackpressureBuffer::new(self, buffer_strategy, capacity)
    }

    fn on_backpressure_drop(self) -> FlowOnBackpressureDrop<Self, Subscription, Item, Error>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
    {
        FlowOnBackpressureDrop::new(self)
    }

    fn on_backpressure_error(self) -> FlowOnBackpressureError<Self, Subscription, Item, Error>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
    {
        FlowOnBackpressureError::new(self)
    }

    fn on_backpressure_latest(self) -> FlowOnBackpressureLatest<Self, Subscription, Item, Error>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
    {
        FlowOnBackpressureLatest::new(self)
    }

    fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> FlowScan<Self, Subscription, Item, Error, ItemOut, BinaryOp>
    where
        Self: Sized,
        ItemOut: Clone,
        BinaryOp: FnMut(ItemOut, Item) -> ItemOut,
    {
        FlowScan::new(self, initial_value, binary_op)
    }

    fn subscribe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> FlowSubscribeOn<Self, Subscription, Item, Error, Scheduler>
    where
        Self: Sized,
        Item: Send + 'static,
        Error: Send + 'static,
        Scheduler: core::Scheduler + Send + 'static,
    {
        FlowSubscribeOn::new(self, scheduler)
    }
}
