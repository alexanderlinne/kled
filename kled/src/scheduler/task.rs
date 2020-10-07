use super::delay_channel::DelayedTask;

pub struct Task {
    boxed_fn: Box<dyn FnOnce() + Send + 'static>,
}

impl Task {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            boxed_fn: Box::new(f),
        }
    }

    pub fn execute(self) {
        (self.boxed_fn)()
    }

    #[cfg(test)]
    pub fn dummy() -> Self {
        Self::new(|| {})
    }
}

impl From<DelayedTask> for Task {
    fn from(delay_task: DelayedTask) -> Self {
        delay_task.task()
    }
}
