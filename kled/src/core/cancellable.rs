/// Base trait for every [`LocalObservable::Cancellable`] and
/// [`SharedObservable::Cancellable`].
///
/// [`LocalObservable::Cancellable`]: trait.LocalObservable.html#associatedtype.Cancellable
/// [`SharedObservable::Cancellable`]: trait.SharedObservable.html#associatedtype.Cancellable
#[blanket(derive(Ref, Box, Rc))]
pub trait Cancellable: Clone {
    /// Cancels the observable the given suscription was provided by.
    fn cancel(&self);
}
