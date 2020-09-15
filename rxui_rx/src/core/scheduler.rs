pub trait Scheduler: Clone {
    fn schedule<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static;

    fn join(&self);
}
