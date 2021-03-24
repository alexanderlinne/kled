use async_trait::async_trait;

#[async_trait]
pub trait Observer<Cancellable, Item, Error> {
    async fn on_subscribe(&mut self, cancellable: Cancellable);
    async fn on_next(&mut self, item: Item);
    async fn on_error(&mut self, error: Error);
    async fn on_completed(&mut self);
}

#[async_trait]
impl<T, Cancellable, Item, Error> Observer<Cancellable, Item, Error> for Box<T>
where
    T: Observer<Cancellable, Item, Error> + Send,
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn on_subscribe(&mut self, cancellable: Cancellable) {
        self.as_mut().on_subscribe(cancellable).await
    }
    async fn on_next(&mut self, item: Item) {
        self.as_mut().on_next(item).await
    }
    async fn on_error(&mut self, error: Error) {
        self.as_mut().on_error(error).await
    }
    async fn on_completed(&mut self) {
        self.as_mut().on_completed().await
    }
}
