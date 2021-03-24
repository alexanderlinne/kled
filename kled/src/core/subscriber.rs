use crate::flow;
use async_trait::async_trait;

#[async_trait]
pub trait Subscriber<Subscription, Item, Error> {
    async fn on_subscribe(&mut self, subscription: Subscription);
    async fn on_next(&mut self, item: Item);
    async fn on_error(&mut self, error: flow::Error<Error>);
    async fn on_completed(&mut self);
}

#[async_trait]
impl<T, Subscription, Item, Error> Subscriber<Subscription, Item, Error> for Box<T>
where
    T: Subscriber<Subscription, Item, Error> + Send,
    Subscription: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    async fn on_subscribe(&mut self, subscription: Subscription) {
        self.as_mut().on_subscribe(subscription).await
    }
    async fn on_next(&mut self, item: Item) {
        self.as_mut().on_next(item).await
    }
    async fn on_error(&mut self, error: flow::Error<Error>) {
        self.as_mut().on_error(error).await
    }
    async fn on_completed(&mut self) {
        self.as_mut().on_completed().await
    }
}
