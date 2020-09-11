use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Base trait for every [`LocalObservable::Subscription`] and
/// [`SharedObservable::Subscription`].
///
/// [`LocalObservable::Subscription`]: trait.LocalObservable.html#associatedtype.Subscription
/// [`SharedObservable::Subscription`]: trait.SharedObservable.html#associatedtype.Subscription
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
        *self.cancelled.borrow()
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

#[derive(Clone)]
enum EitherCancellable<Left, Right> {
    Left(Left),
    Right(Right),
}

impl<Left, Right> Cancellable for EitherCancellable<Left, Right>
where
    Left: Cancellable,
    Right: Cancellable,
{
    fn cancel(&self) {
        match &self {
            Self::Left(cancellable) => cancellable.cancel(),
            Self::Right(cancellable) => cancellable.cancel(),
        }
    }

    fn is_cancelled(&self) -> bool {
        match &self {
            Self::Left(cancellable) => cancellable.is_cancelled(),
            Self::Right(cancellable) => cancellable.is_cancelled(),
        }
    }
}

#[derive(Clone)]
pub struct LocalEitherCancellable<Left, Right> {
    data: Rc<RefCell<EitherCancellable<Left, Right>>>,
}

impl<Left, Right> LocalEitherCancellable<Left, Right> {
    pub fn from_left(left: Left) -> Self {
        Self {
            data: Rc::new(RefCell::new(EitherCancellable::Left(left))),
        }
    }

    pub fn from_right(right: Right) -> Self {
        Self {
            data: Rc::new(RefCell::new(EitherCancellable::Right(right))),
        }
    }

    pub fn set_left(&mut self, left: Left) {
        *self.data.borrow_mut() = EitherCancellable::Left(left);
    }

    pub fn set_right(&mut self, right: Right) {
        *self.data.borrow_mut() = EitherCancellable::Right(right);
    }
}

impl<Left, Right> Cancellable for LocalEitherCancellable<Left, Right>
where
    Left: Cancellable,
    Right: Cancellable,
{
    fn cancel(&self) {
        self.data.borrow().cancel()
    }

    fn is_cancelled(&self) -> bool {
        self.data.borrow().is_cancelled()
    }
}

#[derive(Clone)]
pub struct SharedEitherCancellable<Left, Right> {
    data: Arc<Mutex<EitherCancellable<Left, Right>>>,
}

impl<Left, Right> SharedEitherCancellable<Left, Right> {
    pub fn from_left(left: Left) -> Self {
        Self {
            data: Arc::new(Mutex::new(EitherCancellable::Left(left))),
        }
    }

    pub fn from_right(right: Right) -> Self {
        Self {
            data: Arc::new(Mutex::new(EitherCancellable::Right(right))),
        }
    }

    pub fn set_left(&mut self, left: Left) {
        *self.data.lock().unwrap() = EitherCancellable::Left(left);
    }

    pub fn set_right(&mut self, right: Right) {
        *self.data.lock().unwrap() = EitherCancellable::Right(right);
    }
}

impl<Left, Right> Cancellable for SharedEitherCancellable<Left, Right>
where
    Left: Cancellable,
    Right: Cancellable,
{
    fn cancel(&self) {
        self.data.lock().unwrap().cancel()
    }

    fn is_cancelled(&self) -> bool {
        self.data.lock().unwrap().is_cancelled()
    }
}
