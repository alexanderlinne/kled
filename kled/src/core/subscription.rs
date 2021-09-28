use crate::Never;
use async_trait::async_trait;

#[async_trait]
pub trait Subscription {
    async fn cancel(&self);
    async fn is_cancelled(&self) -> bool;
    async fn request(&self, count: usize);
}

#[async_trait]
impl Subscription for Never {
    async fn cancel(&self) {
        unreachable! {};
    }

    async fn is_cancelled(&self) -> bool {
        unreachable! {};
    }

    async fn request(&self, _: usize) {
        unreachable! {};
    }
}
