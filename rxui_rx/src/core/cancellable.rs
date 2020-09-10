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
    fn cancel(&self);

    /// Returns true, if the observable has been cancelled
    fn is_cancelled(&self) -> bool;
}

#[derive(Clone)]
pub struct LocalCancellable {
    cancelled: Rc<RefCell<bool>>,
}

impl Default for LocalCancellable {
    fn default() -> Self {
        Self {
            cancelled: Rc::new(RefCell::new(false)),
        }
    }
}

impl Cancellable for LocalCancellable {
    fn cancel(&self) {
        *self.cancelled.borrow_mut() = true;
    }

    fn is_cancelled(&self) -> bool {
        *self.cancelled.borrow_mut()
    }
}

#[derive(Clone)]
pub struct SharedCancellable {
    cancelled: Arc<AtomicBool>,
}

impl Default for SharedCancellable {
    fn default() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Cancellable for SharedCancellable {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}
