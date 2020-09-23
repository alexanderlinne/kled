use crate::core;
use crate::flow;
use crate::marker;
use crate::operators;

#[derive(Clone)]
pub struct Flow<Actual> {
    pub(crate) actual: Actual,
}

impl<Actual> Flow<Actual> {
    pub fn new(actual: Actual) -> Self {
        Self { actual }
    }

    pub fn into_shared(self) -> marker::Shared<Self>
    where
        Self: Sized,
    {
        marker::Shared::new(self)
    }

    pub fn on_backpressure_buffer<'o>(
        self,
        buffer_strategy: flow::BufferStrategy,
    ) -> marker::Flow<operators::local::FlowOnBackpressureBuffer<'o, Actual>>
    where
        Actual: core::LocalFlow<'o> + Sized,
    {
        marker::Flow::new(operators::local::FlowOnBackpressureBuffer::new(
            self.actual,
            buffer_strategy,
            flow::default_buffer_capacity(),
        ))
    }

    pub fn on_backpressure_buffer_with_capacity<'o>(
        self,
        buffer_strategy: flow::BufferStrategy,
        buffer_capacity: usize,
    ) -> marker::Flow<operators::local::FlowOnBackpressureBuffer<'o, Actual>>
    where
        Actual: core::LocalFlow<'o> + Sized,
    {
        marker::Flow::new(operators::local::FlowOnBackpressureBuffer::new(
            self.actual,
            buffer_strategy,
            buffer_capacity,
        ))
    }

    pub fn on_backpressure_drop<'o>(
        self,
    ) -> marker::Flow<operators::local::FlowOnBackpressureDrop<'o, Actual>>
    where
        Actual: core::LocalFlow<'o> + Sized,
    {
        marker::Flow::new(operators::local::FlowOnBackpressureDrop::new(self.actual))
    }

    pub fn on_backpressure_error<'o>(
        self,
    ) -> marker::Flow<operators::local::FlowOnBackpressureError<'o, Actual>>
    where
        Actual: core::LocalFlow<'o> + Sized,
    {
        marker::Flow::new(operators::local::FlowOnBackpressureError::new(self.actual))
    }

    pub fn on_backpressure_latest<'o>(
        self,
    ) -> marker::Flow<operators::local::FlowOnBackpressureLatest<'o, Actual>>
    where
        Actual: core::LocalFlow<'o> + Sized,
    {
        marker::Flow::new(operators::local::FlowOnBackpressureLatest::new(self.actual))
    }

    pub fn scan<'o, ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> marker::Flow<operators::FlowScan<Actual, ItemOut, BinaryOp>>
    where
        Actual: core::LocalFlow<'o> + Sized,
        ItemOut: Clone,
        BinaryOp: FnMut(ItemOut, Actual::Item) -> ItemOut,
    {
        marker::Flow::new(operators::FlowScan::new(
            self.actual,
            initial_value,
            binary_op,
        ))
    }
}

impl<Cancellable, Item, Error, T> core::Subscriber<Cancellable, Item, Error> for Flow<T>
where
    T: core::Subscriber<Cancellable, Item, Error>,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.actual.on_subscribe(cancellable);
    }

    fn on_next(&mut self, item: Item) {
        self.actual.on_next(item);
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        self.actual.on_error(error);
    }

    fn on_completed(&mut self) {
        self.actual.on_completed();
    }
}
