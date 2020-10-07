use crate::time;

pub trait Scheduler: Clone {
    fn schedule<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static;

    fn schedule_delayed<F>(&self, delay: time::Duration, task: F)
    where
        F: FnOnce() + Send + 'static;

    fn join(&self);
}
