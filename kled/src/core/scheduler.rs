use crate::time;
use std::future::Future;

pub trait Scheduler: Clone {
    fn schedule<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static;

    fn schedule_fn<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static;

    fn schedule_delayed<Fut>(&self, delay: time::Duration, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static;

    fn schedule_fn_delayed<F>(&self, delay: time::Duration, task: F)
    where
        F: FnOnce() + Send + 'static;

    fn join(&self);
}
