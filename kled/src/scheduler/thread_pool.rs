use crate::core;
use std::cell::UnsafeCell;
use std::future::Future;

#[chronobreak]
mod mock {
    pub use futures::executor::ThreadPool;
    pub use futures_timer::Delay;
    pub use parking_lot::{Condvar, Mutex};
    pub use std::sync::atomic::{AtomicUsize, Ordering};
    pub use std::sync::Arc;
    pub use std::time;
}
use mock::*;

thread_local! {
    static DATA: UnsafeCell<Option<Arc<Data>>> = UnsafeCell::new(None);
}

#[derive(Clone)]
pub struct ThreadPoolScheduler {
    thread_pool: ThreadPool,
    data: Arc<Data>,
}

impl ThreadPoolScheduler {
    pub fn new(num_threads: usize) -> Self {
        let data = Arc::new(Data {
            join_mutex: Mutex::new(()),
            join_cond: Condvar::new(),
            job_count: AtomicUsize::new(0),
        });
        ThreadPoolScheduler {
            thread_pool: Self::create_thread_pool(num_threads, data.clone()),
            data,
        }
    }

    fn create_thread_pool(num_threads: usize, data: Arc<Data>) -> ThreadPool {
        ThreadPool::builder()
            .pool_size(num_threads)
            .after_start(move |_| {
                DATA.with(|glob| {
                    unsafe { *glob.get() = Some(data.clone()) };
                });
            })
            .create()
            .unwrap()
    }
}

impl Default for ThreadPoolScheduler {
    fn default() -> Self {
        ThreadPoolScheduler::new(num_cpus::get())
    }
}

impl core::Scheduler for ThreadPoolScheduler {
    fn schedule<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.data.job_count.fetch_add(1, Ordering::SeqCst);
        self.thread_pool.spawn_ok(async move {
            future.await;

            let data = DATA.with(|data| unsafe { &*data.get() }.as_ref().unwrap());
            data.job_count.fetch_sub(1, Ordering::SeqCst);
            if !data.has_work() {
                let _lock = data.join_mutex.lock();
                data.join_cond.notify_all();
            }
        })
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
    job_count: AtomicUsize,
}

impl Data {
    fn has_work(&self) -> bool {
        self.job_count.load(Ordering::SeqCst) > 0
    }
}
