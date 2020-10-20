use crate::core;
use async_std::task;
use std::future::Future;

#[chronobreak]
mod mock {
    pub use parking_lot::{Condvar, Mutex};
    pub use std::sync::atomic::{AtomicUsize, Ordering};
    pub use std::sync::Arc;
    pub use std::thread;
    pub use std::time;
}
use mock::*;

#[derive(Clone)]
pub struct NewThreadScheduler {
    data: Arc<Data>,
}

impl Default for NewThreadScheduler {
    fn default() -> Self {
        NewThreadScheduler {
            data: Arc::new(Data {
                join_mutex: Mutex::new(()),
                join_cond: Condvar::new(),
                active_count: AtomicUsize::new(0),
            }),
        }
    }
}

impl core::Scheduler for NewThreadScheduler {
    fn schedule<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.data.active_count.fetch_add(1, Ordering::SeqCst);
        let data = self.data.clone();
        thread::spawn(move || {
            task::block_on(future);
            data.active_count.fetch_sub(1, Ordering::SeqCst);
            if !data.has_work() {
                let _ = data.join_mutex.lock();
                data.join_cond.notify_all();
            }
        });
    }

    fn join(&self) {
        if !self.data.has_work() {
            return;
        }

        let mut lock = self.data.join_mutex.lock();
        while self.data.has_work() {
            self.data.join_cond.wait(&mut lock);
        }
    }
}

struct Data {
    join_mutex: Mutex<()>,
    join_cond: Condvar,
    active_count: AtomicUsize,
}

impl Data {
    fn has_work(&self) -> bool {
        self.active_count.load(Ordering::SeqCst) > 0
    }
}
