use super::delay_channel::*;
use super::task::Task;
use crate::core;
use crate::sync::atomic::{AtomicUsize, Ordering};
use crate::sync::{Arc, Condvar, Mutex};
use crate::thread;
use crate::time;

#[derive(Clone)]
pub struct ThreadPoolScheduler {
    sender: DelaySender,
    data: Arc<Data>,
}

impl ThreadPoolScheduler {
    pub fn new(num_threads: usize) -> Self {
        let (sender, receiver) = unbounded();
        let data = Arc::new(Data {
            receiver,
            join_mutex: Mutex::new(()),
            join_cond: Condvar::new(),
            active_count: AtomicUsize::new(0),
            queued_count: AtomicUsize::new(0),
        });

        for _ in 0..num_threads {
            Self::spawn_pool_worker(data.clone());
        }

        ThreadPoolScheduler { sender, data }
    }

    fn spawn_pool_worker(data: Arc<Data>) {
        thread::spawn(move || loop {
            let message = data.receiver.recv();
            let job = match message {
                Ok(job) => job,
                Err(_) => break,
            };
            data.active_count.fetch_add(1, Ordering::SeqCst);
            data.queued_count.fetch_sub(1, Ordering::SeqCst);

            job.execute();

            data.active_count.fetch_sub(1, Ordering::SeqCst);
            if !data.has_work() {
                data.join_cond.notify_all();
            }
        });
    }

    fn schedule_impl<F>(&self, task: F, delay: Option<time::Duration>)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = if let Some(delay) = delay {
            DelayedTask::with_delay(Task::new(task), delay).expect("Unable to create task!")
        } else {
            Task::new(task).into()
        };
        self.data.queued_count.fetch_add(1, Ordering::SeqCst);
        self.sender
            .send(task)
            .expect("Unable to send job into queue");
    }
}

impl Default for ThreadPoolScheduler {
    fn default() -> Self {
        ThreadPoolScheduler::new(num_cpus::get())
    }
}

impl core::Scheduler for ThreadPoolScheduler {
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
        self.data.join_cond.wait(&mut lock);
    }
}

struct Data {
    receiver: DelayReceiver,
    join_mutex: Mutex<()>,
    join_cond: Condvar,
    active_count: AtomicUsize,
    queued_count: AtomicUsize,
}

impl Data {
    fn has_work(&self) -> bool {
        self.queued_count.load(Ordering::SeqCst) > 0 || self.active_count.load(Ordering::SeqCst) > 0
    }
}
