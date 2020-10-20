use std::future::Future;

pub trait Scheduler: Clone + Send + Sync + 'static {
    fn schedule<Fut>(&self, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static;

    fn join(&self);
}
