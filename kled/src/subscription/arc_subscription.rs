use crate::core;
#[chronobreak]
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
#[chronobreak]
use std::sync::Arc;

pub struct ArcSubscriptionStub {
    data: Arc<Data>,
}

impl Default for ArcSubscriptionStub {
    fn default() -> Self {
        Self {
            data: Arc::new(Data {
                cancelled: AtomicBool::new(false),
                requested: AtomicUsize::new(0),
            }),
        }
    }
}

impl ArcSubscriptionStub {
    pub fn subscription(&self) -> ArcSubscription {
        ArcSubscription {
            data: self.data.clone(),
        }
    }

    pub fn get_and_reset_requested(&self) -> usize {
        self.data.requested.swap(0, Ordering::Relaxed)
    }

    pub fn is_cancelled(&self) -> bool {
        self.data.cancelled.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct ArcSubscription {
    data: Arc<Data>,
}

impl core::Subscription for ArcSubscription {
    fn cancel(&self) {
        self.data.cancelled.store(true, Ordering::Relaxed);
    }

    fn is_cancelled(&self) -> bool {
        self.data.cancelled.load(Ordering::Relaxed)
    }

    fn request(&self, count: usize) {
        self.data.requested.fetch_add(count, Ordering::Relaxed);
    }
}

struct Data {
    cancelled: AtomicBool,
    requested: AtomicUsize,
}
