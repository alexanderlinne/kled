use crate::core;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

#[derive(Clone)]
pub struct ThreadPoolScheduler {
    data: Arc<Mutex<ThreadPool>>,
}

impl ThreadPoolScheduler {
    pub fn new(num_threads: usize) -> Self {
        ThreadPoolScheduler {
            data: Arc::new(Mutex::new(ThreadPool::new(num_threads))),
        }
    }
}

impl Default for ThreadPoolScheduler {
    fn default() -> Self {
        ThreadPoolScheduler {
            data: Arc::new(Mutex::new(ThreadPool::default())),
        }
    }
}

impl core::Scheduler for ThreadPoolScheduler {
    fn schedule<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.data.lock().unwrap().execute(move || {
            task();
        });
    }

    fn join(&self) {
        self.data.lock().unwrap().join();
    }
}
