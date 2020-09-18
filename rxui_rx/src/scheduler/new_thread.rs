use crate::core;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct NewThreadScheduler {
    join_handles: Arc<Mutex<Vec<Option<thread::JoinHandle<()>>>>>,
}

impl Default for NewThreadScheduler {
    fn default() -> Self {
        NewThreadScheduler {
            join_handles: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl core::Scheduler for NewThreadScheduler {
    fn schedule<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.join_handles
            .lock()
            .unwrap()
            .push(Some(thread::spawn(move || {
                task();
            })));
    }

    fn join(&self) {
        (*self.join_handles.lock().unwrap())
            .iter_mut()
            .for_each(|handle| handle.take().expect("Empty handle!").join().unwrap());
    }
}
