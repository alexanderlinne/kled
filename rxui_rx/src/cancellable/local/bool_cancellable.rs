use crate::core;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct BoolCancellable {
    cancelled: Rc<RefCell<bool>>,
}

impl Default for BoolCancellable {
    fn default() -> Self {
        Self {
            cancelled: Rc::new(RefCell::new(false)),
        }
    }
}

impl core::Cancellable for BoolCancellable {
    fn cancel(&self) {
        *self.cancelled.borrow_mut() = true;
    }

    fn is_cancelled(&self) -> bool {
        *self.cancelled.borrow()
    }
}
