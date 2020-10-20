use std::future::Future;
#[chronobreak]
use std::time;

pub trait Scheduler: Clone + Send + Sync + 'static {
    fn schedule<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static;

    fn schedule_delayed<Fut>(&self, delay: time::Duration, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static;

    fn join(&self);
}
