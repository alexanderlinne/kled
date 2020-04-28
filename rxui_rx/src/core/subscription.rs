use crate::core;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct LocalSubscription {
    observed: Rc<RefCell<bool>>,
}

impl LocalSubscription {
    pub fn new(observed: Rc<RefCell<bool>>) -> Self {
        Self { observed }
    }
}

impl core::ObservableSubscription for LocalSubscription {
    fn cancel(self) {
        *self.observed.borrow_mut() = false;
    }
}

pub struct SharedSubscription {
    observed: Arc<AtomicBool>,
}

impl SharedSubscription {
    pub fn new(observed: Arc<AtomicBool>) -> Self {
        Self { observed }
    }
}

impl core::ObservableSubscription for SharedSubscription {
    fn cancel(self) {
        self.observed.store(false, Ordering::Release);
    }
}

/*pub struct LocalRefCountSubscription<Subscription> {
    subscription: Subscription,
    subscription_count: usize
}

impl<Subscription> LocalRefCountSubscription<Subscription> {
    pub fn new(subscription: Subscription) -> Self {
        Self { subscription, subscription_count: 0 }
    }
}*/
