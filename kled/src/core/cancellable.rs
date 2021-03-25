use crate::util;
use async_trait::async_trait;

/// Base trait for every [`LocalObservable::Cancellable`] and
/// [`SharedObservable::Cancellable`].
///
/// [`LocalObservable::Cancellable`]: trait.LocalObservable.html#associatedtype.Cancellable
/// [`SharedObservable::Cancellable`]: trait.SharedObservable.html#associatedtype.Cancellable
#[async_trait]
pub trait Cancellable: Clone {
    /// Cancels the observable the given suscription was provided by.
    async fn cancel(&self);
}

#[async_trait]
impl Cancellable for util::Never {
    async fn cancel(&self) {
        unreachable!{};
    }
}
