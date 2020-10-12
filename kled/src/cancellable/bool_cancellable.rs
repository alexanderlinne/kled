use crate::core;

#[chronobreak]
use std::sync::atomic::{AtomicBool, Ordering};
#[chronobreak]
use std::sync::Arc;

pub struct BoolCancellableStub {
    cancelled: Arc<AtomicBool>,
}

impl Default for BoolCancellableStub {
    fn default() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl BoolCancellableStub {
    pub fn cancellable(&self) -> BoolCancellable {
        BoolCancellable {
            cancelled: self.cancelled.clone(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct BoolCancellable {
    cancelled: Arc<AtomicBool>,
}

impl core::Cancellable for BoolCancellable {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}
