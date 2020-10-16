use crate::core;

#[chronobreak]
use std::sync::atomic::{AtomicBool, Ordering};
#[chronobreak]
use std::sync::Arc;

pub struct ArcCancellableStub {
    cancelled: Arc<AtomicBool>,
}

impl Default for ArcCancellableStub {
    fn default() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl ArcCancellableStub {
    pub fn cancellable(&self) -> ArcCancellable {
        ArcCancellable {
            cancelled: self.cancelled.clone(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct ArcCancellable {
    cancelled: Arc<AtomicBool>,
}

impl core::Cancellable for ArcCancellable {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}
