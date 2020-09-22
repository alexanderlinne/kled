use crate::core;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct AccumulateSubscriptionStub {
    data: Rc<RefCell<Data>>,
}

impl Default for AccumulateSubscriptionStub {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new(Data {
                cancelled: false,
                requested: 0,
            })),
        }
    }
}

impl AccumulateSubscriptionStub {
    pub fn subscription(&self) -> AccumulateSubscription {
        AccumulateSubscription {
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
pub struct AccumulateSubscription {
    data: Rc<RefCell<Data>>,
}

impl core::Subscription for AccumulateSubscription {
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
