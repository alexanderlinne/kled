use crate::core;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub trait Observation {
    fn cancel(self);
}

pub struct LocalObservation {
    observed: Rc<RefCell<bool>>,
}

impl LocalObservation {
    pub(crate) fn new(observed: Rc<RefCell<bool>>) -> Self {
        Self { observed }
    }
}

impl core::Observation for LocalObservation {
    fn cancel(self) {
        *self.observed.borrow_mut() = false;
    }
}

pub struct SharedObservation {
    observed: Arc<AtomicBool>,
}

impl SharedObservation {
    pub(crate) fn new(observed: Arc<AtomicBool>) -> Self {
        Self { observed }
    }
}

impl core::Observation for SharedObservation {
    fn cancel(self) {
        self.observed.store(false, Ordering::Release);
    }
}
