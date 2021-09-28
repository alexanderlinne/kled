use crate::core;
use async_trait::async_trait;
use std::marker::PhantomData;

#[operator(type = "observable", item = "ItemOut")]
pub struct Map<ItemOut, UnaryOp>
where
    UnaryOp: FnMut(Item) -> ItemOut,
{
    unary_op: UnaryOp,
}

#[derive(new)]
struct MapObserver<Observer, ItemOut, UnaryOp> {
    observer: Observer,
    unary_op: UnaryOp,
    phantom: PhantomData<ItemOut>,
}

#[async_trait]
impl<Cancellable, ItemIn, Observer, ItemOut, Error, UnaryOp>
    core::Observer<Cancellable, ItemIn, Error> for MapObserver<Observer, ItemOut, UnaryOp>
where
    Observer: core::Observer<Cancellable, ItemOut, Error> + Send,
    Cancellable: Send + 'static,
    ItemIn: Send + 'static,
    ItemOut: Send,
    Error: Send + 'static,
    UnaryOp: FnMut(ItemIn) -> ItemOut + Send,
{
    async fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.observer.on_subscribe(cancellable).await;
    }

    async fn on_next(&mut self, item: ItemIn) {
        self.observer.on_next((self.unary_op)(item)).await;
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
    async fn map() {
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .map(|a| a + 1)
            .subscribe(test_observer.clone())
            .await;

        assert_eq!(test_observer.status().await, ObserverStatus::Completed);
        assert_eq!(test_observer.items().await, vec![1, 2, 3, 4]);
    }
}
