use crate::core;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

pub struct ThreadPoolScheduler {
    data: Arc<Mutex<Data>>,
}

struct Data {
    threadpool: ThreadPool,
}

impl ThreadPoolScheduler {
    pub fn new(num_threads: usize) -> Self {
        ThreadPoolScheduler {
            data: Arc::new(Mutex::new(Data {
                threadpool: ThreadPool::new(num_threads),
            })),
        }
    }
}

impl Default for ThreadPoolScheduler {
    fn default() -> Self {
        ThreadPoolScheduler {
            data: Arc::new(Mutex::new(Data {
                threadpool: ThreadPool::default(),
            })),
        }
    }
}

impl core::Scheduler for ThreadPoolScheduler {
    type Worker = ThreadPoolWorker;

    fn create_worker(&self) -> ThreadPoolWorker {
        ThreadPoolWorker {
            data: self.data.clone(),
        }
    }

    fn schedule<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        &self.data.lock().unwrap().threadpool.execute(move || {
            task();
        });
    }

    fn join(&self) {
        &self.data.lock().unwrap().threadpool.join();
    }
}

#[derive(Clone)]
pub struct ThreadPoolWorker {
    data: Arc<Mutex<Data>>,
}

impl core::Worker for ThreadPoolWorker {
    fn schedule<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        &self.data.lock().unwrap().threadpool.execute(move || {
            task();
        });
    }
}
