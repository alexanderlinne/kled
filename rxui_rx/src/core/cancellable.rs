use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Base trait for every [`LocalObservable::Subscription`] and
/// [`SharedObservable::Subscription`].
///
/// [`LocalObservable::Subscription`]: trait.LocalObservable.html#associatedtype.Subscription
/// [`SharedObservable::Subscription`]: trait.SharedObservable.html#associatedtype.Subscription
pub trait Cancellable {
    /// Cancels the observable the given suscription was provided by.
    fn cancel(self);
}

pub struct LocalCancellable {
    observed: Rc<RefCell<bool>>,
}

impl LocalCancellable {
    pub fn new(observed: Rc<RefCell<bool>>) -> Self {
        Self { observed }
    }
}

impl Cancellable for LocalCancellable {
    fn cancel(self) {
        *self.observed.borrow_mut() = false;
    }
}

pub struct SharedCancellable {
    observed: Arc<AtomicBool>,
}

impl SharedCancellable {
    pub fn new(observed: Arc<AtomicBool>) -> Self {
        Self { observed }
    }
}

impl Cancellable for SharedCancellable {
    fn cancel(self) {
        self.observed.store(false, Ordering::Release);
    }
}
