use crate::core;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct EitherCancellable<Left, Right> {
    data: Rc<RefCell<Data<Left, Right>>>,
}

impl<Left, Right> EitherCancellable<Left, Right> {
    pub fn from_left(left: Left) -> Self {
        Self {
            data: Rc::new(RefCell::new(Data::Left(left))),
        }
    }

    pub fn from_right(right: Right) -> Self {
        Self {
            data: Rc::new(RefCell::new(Data::Right(right))),
        }
    }

    pub fn set_left(&mut self, left: Left) {
        *self.data.borrow_mut() = Data::Left(left);
    }

    pub fn set_right(&mut self, right: Right) {
        *self.data.borrow_mut() = Data::Right(right);
    }
}

impl<Left, Right> core::Cancellable for EitherCancellable<Left, Right>
where
    Left: core::Cancellable,
    Right: core::Cancellable,
{
    fn cancel(&self) {
        self.data.borrow().cancel()
    }

    fn is_cancelled(&self) -> bool {
        self.data.borrow().is_cancelled()
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
