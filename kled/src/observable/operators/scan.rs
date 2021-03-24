use crate::core;
use async_trait::async_trait;

#[operator(type = "observable", item = "ItemOut")]
pub struct Scan<ItemOut, BinaryOp>
where
    ItemOut: Clone,
    BinaryOp: FnMut(ItemOut, Item) -> ItemOut,
{
    initial_value: ItemOut,
    binary_op: BinaryOp,
}

#[derive(new)]
struct ScanObserver<Observer, ItemOut, BinaryOp> {
    observer: Observer,
    previous_value: ItemOut,
    binary_op: BinaryOp,
}

#[async_trait]
impl<Cancellable, ItemIn, Observer, ItemOut, Error, BinaryOp>
    core::Observer<Cancellable, ItemIn, Error> for ScanObserver<Observer, ItemOut, BinaryOp>
where
    Observer: core::Observer<Cancellable, ItemOut, Error> + Send,
    BinaryOp: FnMut(ItemOut, ItemIn) -> ItemOut + Send,
    Cancellable: Send + 'static,
    ItemIn: Send + 'static,
    ItemOut: Clone + Send,
    Error: Send + 'static,
{
    async fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.observer.on_subscribe(cancellable).await;
        let value = self.previous_value.clone();
        self.observer.on_next(value).await;
    }

    async fn on_next(&mut self, item: ItemIn) {
        self.previous_value = (self.binary_op)(self.previous_value.clone(), item);
        let value = self.previous_value.clone();
        self.observer.on_next(value).await;
    }

    async fn on_error(&mut self, error: Error) {
        self.observer.on_error(error).await;
    }

    async fn on_completed(&mut self) {
        self.observer.on_completed().await;
    }
}

#[cfg(test)]
mod tests {
    use crate::observer::*;
    use crate::prelude::*;

    #[async_std::test]
    async fn local_scan() {
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .scan(0, |a, b| a + b)
            .subscribe(test_observer.clone()).await;

        assert_eq!(test_observer.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer.items().await, vec![0, 0, 1, 3, 6]);
    }
}
