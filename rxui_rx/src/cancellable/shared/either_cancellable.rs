use crate::core;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct EitherCancellable<Left, Right> {
    data: Arc<Mutex<Data<Left, Right>>>,
}

impl<Left, Right> EitherCancellable<Left, Right> {
    pub fn from_left(left: Left) -> Self {
        Self {
            data: Arc::new(Mutex::new(Data::Left(left))),
        }
    }

    pub fn from_right(right: Right) -> Self {
        Self {
            data: Arc::new(Mutex::new(Data::Right(right))),
        }
    }

    pub fn set_left(&mut self, left: Left) {
        *self.data.lock().unwrap() = Data::Left(left);
    }

    pub fn set_right(&mut self, right: Right) {
        *self.data.lock().unwrap() = Data::Right(right);
    }
}

impl<Left, Right> core::Cancellable for EitherCancellable<Left, Right>
where
    Left: core::Cancellable,
    Right: core::Cancellable,
{
    fn cancel(&self) {
        self.data.lock().unwrap().cancel()
    }

    fn is_cancelled(&self) -> bool {
        self.data.lock().unwrap().is_cancelled()
    }
}

#[derive(Clone)]
enum Data<Left, Right> {
    Left(Left),
    Right(Right),
}

impl<Left, Right> core::Cancellable for Data<Left, Right>
where
    Left: core::Cancellable,
    Right: core::Cancellable,
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
