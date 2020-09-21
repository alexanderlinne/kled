use crate::core;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct IgnoreSubscriptionStub {
    data: Arc<AtomicBool>,
}

impl Default for IgnoreSubscriptionStub {
    fn default() -> Self {
        Self {
            data: Arc::new(AtomicBool::new(false)),
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
        self.data.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct IgnoreSubscription {
    data: Arc<AtomicBool>,
}

impl core::Subscription for IgnoreSubscription {
    fn cancel(&self) {
        self.data.store(true, Ordering::Relaxed);
    }

    fn is_cancelled(&self) -> bool {
        self.data.load(Ordering::Relaxed)
    }

    fn request(&self, _: usize) {}
}
