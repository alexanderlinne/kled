use crate::core;
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
