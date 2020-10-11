use crate::core;
use std::marker::PhantomData;

#[derive(new)]
pub struct ObservableScan<Observable, Cancellable, Item, Error, ItemOut, BinaryOp>
where
    Observable: core::Observable<Cancellable, Item, Error>,
    ItemOut: Clone,
    BinaryOp: FnMut(ItemOut, Item) -> ItemOut,
{
    observable: Observable,
    initial_value: ItemOut,
    binary_op: BinaryOp,
    phantom: PhantomData<(Cancellable, Item, Error)>,
}

impl<Observable, Cancellable, Item, Error, ItemOut, BinaryOp>
    core::Observable<Cancellable, ItemOut, Error>
    for ObservableScan<Observable, Cancellable, Item, Error, ItemOut, BinaryOp>
where
    Observable: core::Observable<Cancellable, Item, Error>,
    Cancellable: core::Cancellable,
    BinaryOp: FnMut(ItemOut, Item) -> ItemOut + Send + 'static,
    ItemOut: Clone + Send + 'static,
{
    fn subscribe<Downstream>(self, downstream: Downstream)
    where
        Downstream: core::Observer<Cancellable, ItemOut, Error> + Send + 'static,
    {
        self.observable.subscribe(ScanObserver::new(
            downstream,
            self.initial_value,
            self.binary_op,
        ));
    }
}

#[derive(new)]
struct ScanObserver<Observer, ItemOut, BinaryOp> {
    observer: Observer,
    previous_value: ItemOut,
    binary_op: BinaryOp,
}

impl<Cancellable, ItemIn, Observer, ItemOut, Error, BinaryOp>
    core::Observer<Cancellable, ItemIn, Error> for ScanObserver<Observer, ItemOut, BinaryOp>
where
    Observer: core::Observer<Cancellable, ItemOut, Error>,
    BinaryOp: FnMut(ItemOut, ItemIn) -> ItemOut,
    ItemOut: Clone,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.observer.on_subscribe(cancellable);
    }
    fn on_next(&mut self, item: ItemIn) {
        self.previous_value = (self.binary_op)(self.previous_value.clone(), item);
        self.observer.on_next(self.previous_value.clone());
    }
    fn on_error(&mut self, error: Error) {
        self.observer.on_error(error);
    }
    fn on_completed(&mut self) {
        self.observer.on_completed();
    }
}

#[cfg(test)]
mod tests {
    use crate::observer::*;
    use crate::prelude::*;

    #[test]
    fn local_scan() {
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .scan(0, |a, b| a + b)
            .subscribe(test_observer.clone());

        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0, 1, 3, 6]);
    }
}
