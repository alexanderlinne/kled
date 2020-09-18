use crate::core;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct BoolSubscriptionStub {
    data: Rc<RefCell<Data>>,
}

impl Default for BoolSubscriptionStub {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new(Data {
                cancelled: false,
                requested: 0,
            })),
        }
    }
}

impl BoolSubscriptionStub {
    pub fn subscription(&self) -> BoolSubscription {
        BoolSubscription {
            data: self.data.clone(),
        }
    }

    pub fn get_and_reset_requested(&self) -> usize {
        let mut data = self.data.borrow_mut();
        let result = data.requested;
        data.requested = 0;
        result
    }

    pub fn is_cancelled(&self) -> bool {
        (*self.data.borrow()).cancelled
    }
}

#[derive(Clone)]
pub struct BoolSubscription {
    data: Rc<RefCell<Data>>,
}

impl core::Subscription for BoolSubscription {
    fn cancel(&self) {
        (*self.data.borrow_mut()).cancelled = true;
    }

    fn is_cancelled(&self) -> bool {
        (*self.data.borrow()).cancelled
    }

    fn request(&self, count: usize) {
        (*self.data.borrow_mut()).requested += count;
    }
}

#[derive(Clone)]
struct Data {
    cancelled: bool,
    requested: usize,
}
