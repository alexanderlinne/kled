use crate::core;
use crate::sync::atomic::{AtomicUsize, Ordering};
use crate::sync::{Arc, Condvar, Mutex};
use crate::thread;
use crate::time;

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

impl NewThreadScheduler {
    fn schedule_impl<F>(&self, task: F, delay: Option<time::Duration>)
    where
        F: FnOnce() + Send + 'static,
    {
        self.data.active_count.fetch_add(1, Ordering::SeqCst);
        let data = self.data.clone();
        thread::spawn(move || {
            if let Some(delay) = delay {
                thread::sleep(delay);
            }
            task();
            data.active_count.fetch_sub(1, Ordering::SeqCst);
            if !data.has_work() {
                let _ = data.join_mutex.lock();
                data.join_cond.notify_all();
            }
        });
    }
}

impl core::Scheduler for NewThreadScheduler {
    fn schedule<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.schedule_impl(task, None)
    }

    fn schedule_delayed<F>(&self, delay: time::Duration, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.schedule_impl(task, Some(delay))
    }

    fn join(&self) {
        if !self.data.has_work() {
            return;
        }

        let mut lock = self.data.join_mutex.lock();
        if self.data.has_work() {
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
