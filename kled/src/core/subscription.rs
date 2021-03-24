use async_trait::async_trait;

#[async_trait]
pub trait Subscription {
    async fn cancel(&self);
    async fn is_cancelled(&self) -> bool;
    async fn request(&self, count: usize);
}
