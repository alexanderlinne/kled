use crate::core;
use crate::sync::atomic::{AtomicBool, Ordering};
use crate::sync::Arc;

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

impl core::CancellableProvider for BoolCancellableStub {
    type Cancellable = BoolCancellable;

    fn cancellable(&self) -> BoolCancellable {
        BoolCancellable {
            cancelled: self.cancelled.clone(),
        }
    }
}

impl BoolCancellableStub {
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
