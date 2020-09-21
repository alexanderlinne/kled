use crate::core;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct IgnoreSubscriptionStub {
    data: Rc<RefCell<bool>>,
}

impl Default for IgnoreSubscriptionStub {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new(false)),
        }
    }
}

impl IgnoreSubscriptionStub {
    pub fn subscription(&self) -> IgnoreSubscription {
        IgnoreSubscription {
            data: self.data.clone(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        *self.data.borrow()
    }
}

#[derive(Clone)]
pub struct IgnoreSubscription {
    data: Rc<RefCell<bool>>,
}

impl core::Subscription for IgnoreSubscription {
    fn cancel(&self) {
        *self.data.borrow_mut() = true;
    }

    fn is_cancelled(&self) -> bool {
        *self.data.borrow()
    }

    fn request(&self, _: usize) {}
}
