use crate::core;
use crate::sync::atomic::{AtomicBool, Ordering};
use crate::sync::Arc;

#[derive(Clone)]
pub struct BoolCancellable {
    cancelled: Arc<AtomicBool>,
}

impl Default for BoolCancellable {
    fn default() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl core::Cancellable for BoolCancellable {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}
