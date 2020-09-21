use crate::core;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LambdaSubscriptionStub {
    data: Arc<Data>,
}

impl LambdaSubscriptionStub {
    pub fn new<RequestFn>(request_fn: RequestFn) -> Self
    where
        RequestFn: Fn(usize) + Send + 'static,
    {
        Self {
            data: Arc::new(Data {
                cancelled: AtomicBool::new(false),
                request_fn: Mutex::new(Box::new(request_fn)),
            }),
        }
    }
}

impl LambdaSubscriptionStub {
    pub fn subscription(&self) -> LambdaSubscription {
        LambdaSubscription {
            data: self.data.clone(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.data.cancelled.load(Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct LambdaSubscription {
    data: Arc<Data>,
}

impl core::Subscription for LambdaSubscription {
    fn cancel(&self) {
        self.data.cancelled.store(true, Ordering::Relaxed);
    }

    fn is_cancelled(&self) -> bool {
        self.data.cancelled.load(Ordering::Relaxed)
    }

    fn request(&self, count: usize) {
        (self.data.request_fn.lock().unwrap())(count);
    }
}

struct Data {
    cancelled: AtomicBool,
    request_fn: Mutex<Box<dyn Fn(usize) + Send + 'static>>,
}
