use crate::core;
use crate::flow;
use crate::operators::*;

pub trait Flow {
    type Item;
    type Error;
    type Subscription: core::Subscription + Send + Sync + 'static;

    fn actual_subscribe<Subscriber>(self, subscriber: Subscriber)
    where
        Subscriber: core::Subscriber<Self::Subscription, Self::Item, Self::Error> + Send + 'static;

    fn map<ItemOut, UnaryOp>(self, unary_op: UnaryOp) -> FlowMap<Self, ItemOut, UnaryOp>
    where
        Self: Sized,
        UnaryOp: FnMut(Self::Item) -> ItemOut,
    {
        FlowMap::new(self, unary_op)
    }

    fn observe_on<Scheduler>(self, scheduler: Scheduler) -> FlowObserveOn<Self, Scheduler>
    where
        Self: Sized,
        Self::Item: Send,
        Self::Error: Send,
        Scheduler: core::Scheduler + Send + 'static,
    {
        FlowObserveOn::new(self, scheduler)
    }

    fn on_backpressure_buffer(
        self,
        buffer_strategy: flow::BufferStrategy,
    ) -> FlowOnBackpressureBuffer<Self>
    where
        Self: Sized,
        Self::Item: Send + 'static,
        Self::Error: Send + 'static,
    {
        FlowOnBackpressureBuffer::new(self, buffer_strategy, flow::default_buffer_capacity())
    }

    fn on_backpressure_buffer_with_capacity(
        self,
        buffer_strategy: flow::BufferStrategy,
        capacity: usize,
    ) -> FlowOnBackpressureBuffer<Self>
    where
        Self: Sized,
        Self::Item: Send + 'static,
        Self::Error: Send + 'static,
    {
        FlowOnBackpressureBuffer::new(self, buffer_strategy, capacity)
    }

    fn on_backpressure_drop(self) -> FlowOnBackpressureDrop<Self>
    where
        Self: Sized,
        Self::Item: Send + 'static,
        Self::Error: Send + 'static,
    {
        FlowOnBackpressureDrop::new(self)
    }

    fn on_backpressure_error(self) -> FlowOnBackpressureError<Self>
    where
        Self: Sized,
        Self::Item: Send + 'static,
        Self::Error: Send + 'static,
    {
        FlowOnBackpressureError::new(self)
    }

    fn on_backpressure_latest(self) -> FlowOnBackpressureLatest<Self>
    where
        Self: Sized,
        Self::Item: Send + 'static,
        Self::Error: Send + 'static,
    {
        FlowOnBackpressureLatest::new(self)
    }

    fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> FlowScan<Self, ItemOut, BinaryOp>
    where
        Self: Sized,
        ItemOut: Clone,
        BinaryOp: FnMut(ItemOut, Self::Item) -> ItemOut,
    {
        FlowScan::new(self, initial_value, binary_op)
    }
}
