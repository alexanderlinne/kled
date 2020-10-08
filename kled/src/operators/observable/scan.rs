use crate::core;

#[derive(new, reactive_operator)]
pub struct ObservableScan<Observable, ItemOut, BinaryOp>
where
    Observable: core::Observable,
    ItemOut: Clone,
    BinaryOp: FnMut(ItemOut, Observable::Item) -> ItemOut,
{
    #[upstream(item = "ItemOut")]
    observable: Observable,
    initial_value: ItemOut,
    binary_op: BinaryOp,
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
