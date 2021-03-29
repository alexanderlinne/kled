use crate::Never;
use async_trait::async_trait;

/// Base trait which must be implemented for every cancellable an observable
/// emits.
#[async_trait]
pub trait Cancellable: Clone {
    /// Cancels the observable. While the implementation must guarantee that
    /// the observable stops emitting in a timely fashion, it may emit further
    /// signals after the call to cancel.
    async fn cancel(&self);
}

#[async_trait]
impl Cancellable for Never {
    async fn cancel(&self) {
        unreachable!{};
    }
}
