use crate::core;
use crate::flow;
use crate::marker;
use crate::operators;

#[derive(Clone)]
pub struct Shared<Actual> {
    pub(crate) actual: Actual,
}

impl<Actual> Shared<Actual> {
    pub fn new(actual: Actual) -> Self {
        Self { actual }
    }
}

impl<Observable> Shared<Observable>
where
    Observable: core::Observable,
{
    pub fn observe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> Shared<operators::ObservableObserveOn<Observable, Scheduler>>
    where
        Self: Sized,
        Observable: core::SharedObservable,
        Observable::Cancellable: Send,
        Observable::Item: Send,
        Observable::Error: Send,
        Scheduler: core::Scheduler + Send,
    {
        Shared::new(operators::ObservableObserveOn::new(self.actual, scheduler))
    }

    pub fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> Shared<operators::ObservableScan<Observable, ItemOut, BinaryOp>>
    where
        Self: Sized,
        ItemOut: Clone,
        BinaryOp: FnMut(ItemOut, Observable::Item) -> ItemOut,
    {
        Shared::new(operators::ObservableScan::new(
            self.actual,
            initial_value,
            binary_op,
        ))
    }

    pub fn subscribe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> Shared<operators::ObservableSubscribeOn<Observable, Scheduler>>
    where
        Self: Sized,
        Scheduler: core::Scheduler + Send,
    {
        Shared::new(operators::ObservableSubscribeOn::new(
            self.actual,
            scheduler,
        ))
    }
}

impl<Actual> Shared<marker::Flow<Actual>>
where
    Actual: core::Flow,
    Actual::Item: Send,
    Actual::Error: Send,
{
    pub fn observe_on<Scheduler>(
        self,
        scheduler: Scheduler,
    ) -> Shared<marker::Flow<operators::FlowObserveOn<Actual, Scheduler>>>
    where
        Self: Sized,
        Actual: core::SharedFlow,
        Scheduler: core::Scheduler + Send,
    {
        Shared::new(marker::Flow::new(operators::FlowObserveOn::new(
            self.actual.actual,
            scheduler,
        )))
    }

    pub fn on_backpressure_buffer(
        self,
        buffer_strategy: flow::BufferStrategy,
    ) -> marker::Shared<marker::Flow<operators::shared::FlowOnBackpressureBuffer<Actual>>>
    where
        Actual: core::SharedFlow + Sized,
    {
        marker::Shared::new(marker::Flow::new(
            operators::shared::FlowOnBackpressureBuffer::new(
                self.actual.actual,
                buffer_strategy,
                flow::default_buffer_capacity(),
            ),
        ))
    }

    pub fn on_backpressure_buffer_with_capacity(
        self,
        buffer_strategy: flow::BufferStrategy,
        buffer_capacity: usize,
    ) -> marker::Shared<marker::Flow<operators::shared::FlowOnBackpressureBuffer<Actual>>>
    where
        Actual: core::SharedFlow + Sized,
    {
        marker::Shared::new(marker::Flow::new(
            operators::shared::FlowOnBackpressureBuffer::new(
                self.actual.actual,
                buffer_strategy,
                buffer_capacity,
            ),
        ))
    }

    pub fn on_backpressure_drop(
        self,
    ) -> marker::Shared<marker::Flow<operators::shared::FlowOnBackpressureDrop<Actual>>>
    where
        Actual: core::SharedFlow + Sized,
    {
        marker::Shared::new(marker::Flow::new(
            operators::shared::FlowOnBackpressureDrop::new(self.actual.actual),
        ))
    }

    pub fn on_backpressure_error(
        self,
    ) -> marker::Shared<marker::Flow<operators::shared::FlowOnBackpressureError<Actual>>>
    where
        Actual: core::SharedFlow + Sized,
    {
        marker::Shared::new(marker::Flow::new(
            operators::shared::FlowOnBackpressureError::new(self.actual.actual),
        ))
    }

    pub fn on_backpressure_latest(
        self,
    ) -> marker::Shared<marker::Flow<operators::shared::FlowOnBackpressureLatest<Actual>>>
    where
        Actual: core::SharedFlow + Sized,
    {
        marker::Shared::new(marker::Flow::new(
            operators::shared::FlowOnBackpressureLatest::new(self.actual.actual),
        ))
    }

    pub fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> marker::Shared<marker::Flow<operators::FlowScan<Actual, ItemOut, BinaryOp>>>
    where
        Actual: core::SharedFlow + Sized,
        ItemOut: Clone,
        BinaryOp: FnMut(ItemOut, Actual::Item) -> ItemOut,
    {
        marker::Shared::new(marker::Flow::new(operators::FlowScan::new(
            self.actual.actual,
            initial_value,
            binary_op,
        )))
    }
}

impl<Cancellable, Item, Error, T> core::Observer<Cancellable, Item, Error> for Shared<T>
where
    T: core::Observer<Cancellable, Item, Error>,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.actual.on_subscribe(cancellable);
    }

    fn on_next(&mut self, item: Item) {
        self.actual.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        self.actual.on_error(error);
    }

    fn on_completed(&mut self) {
        self.actual.on_completed();
    }
}
