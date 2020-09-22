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

    pub fn observe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> marker::Shared<marker::Flow<operators::FlowObserveOn<Actual, Scheduler>>>
    where
        Actual: core::SharedFlow + Sized,
        Actual::Subscription: Send,
        Actual::Item: Send,
        Actual::Error: Send,
        Scheduler: core::Scheduler + Send,
    {
        marker::Shared::new(marker::Flow::new(operators::FlowObserveOn::new(
            self.actual,
            scheduler,
        )))
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
