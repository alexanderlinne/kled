use std::future::Future;
#[chronobreak]
use std::time;
use crate::scheduler;

pub trait Scheduler: Clone + Send + Sync + 'static {
    fn schedule<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static;

    fn schedule_delayed<Fut>(&self, delay: time::Duration, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static;

    fn worker<State>(&self, state: State) -> scheduler::Worker<State>
    where
        State: Send + 'static,
    {
        scheduler::Worker::new(self.clone(), state)
    }

    fn join(&self);
}
