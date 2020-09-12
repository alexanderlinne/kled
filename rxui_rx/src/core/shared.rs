use crate::core;
use crate::operators;

#[derive(Clone)]
pub struct Shared<Observable> {
    pub(crate) actual_observable: Observable,
}

impl<Observable> Shared<Observable> {
    pub fn new(actual_observable: Observable) -> Self {
        Self { actual_observable }
    }

    pub fn observe_on<Scheduler>(
        self,
        scheduler: &Scheduler,
    ) -> Shared<operators::ObservableObserveOn<Observable, Scheduler::Worker>>
    where
        Self: Sized,
        Scheduler: core::Scheduler,
    {
        Shared::new(operators::ObservableObserveOn::new(
            self.actual_observable,
            scheduler.create_worker(),
        ))
    }

    pub fn scan<ItemOut, BinaryOp>(
        self,
        initial_value: ItemOut,
        binary_op: BinaryOp,
    ) -> Shared<operators::ObservableScan<Observable, ItemOut, BinaryOp>>
    where
        Self: Sized,
        Observable: core::Observable,
        ItemOut: Clone,
        BinaryOp: FnMut(ItemOut, Observable::Item) -> ItemOut,
    {
        Shared::new(operators::ObservableScan::new(
            self.actual_observable,
            initial_value,
            binary_op,
        ))
    }

    pub fn subscribe_on<Scheduler>(
        self,
        scheduler: &Scheduler,
    ) -> Shared<operators::ObservableSubscribeOn<Observable, Scheduler::Worker>>
    where
        Self: Sized,
        Scheduler: core::Scheduler,
    {
        Shared::new(operators::ObservableSubscribeOn::new(
            self.actual_observable,
            scheduler.create_worker(),
        ))
    }
}

impl<Cancellable, Item, Error, T> core::Observer<Cancellable, Item, Error> for Shared<T>
where
    T: core::Observer<Cancellable, Item, Error>,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.actual_observable.on_subscribe(cancellable);
    }

    fn on_next(&mut self, item: Item) {
        self.actual_observable.on_next(item);
    }

    fn on_error(&mut self, error: Error) {
        self.actual_observable.on_error(error);
    }

    fn on_completed(&mut self) {
        self.actual_observable.on_completed();
    }
}
