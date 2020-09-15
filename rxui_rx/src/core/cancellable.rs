/// Base trait for every [`LocalObservable::Cancellable`] and
/// [`SharedObservable::Cancellable`].
///
/// [`LocalObservable::Cancellable`]: trait.LocalObservable.html#associatedtype.Cancellable
/// [`SharedObservable::Cancellable`]: trait.SharedObservable.html#associatedtype.Cancellable
pub trait Cancellable: Clone {
    /// Cancels the observable the given suscription was provided by.
    fn cancel(&self);

    /// Returns true, if the observable has been cancelled
    fn is_cancelled(&self) -> bool;
}

pub trait CancellableProvider {
    type Cancellable: Cancellable;

    fn cancellable(&self) -> Self::Cancellable;
}
