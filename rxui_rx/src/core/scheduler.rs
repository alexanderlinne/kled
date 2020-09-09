pub trait Scheduler {
    type Worker: Worker + Send + 'static;

    fn create_worker(&self) -> Self::Worker;

    fn schedule<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static;

    fn join(&self);
}

pub trait Worker: Clone {
    fn schedule<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static;
}
