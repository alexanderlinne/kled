use crate::core;
use futures::prelude::*;
#[chronobreak]
use std::time;
use super::delay_channel::*;
use futures::executor;

pub struct Worker<State> {
    sender: DelaySender<Box<dyn FnOnce(&mut State) + Send + 'static>>,
}

impl<State> Worker<State> {
    pub fn new<Scheduler>(scheduler: Scheduler, mut state: State) -> Self
    where
        Scheduler: core::Scheduler,
        State: Send + 'static,
    {
        let (sender, mut receiver) = unbounded();
        scheduler.schedule(async move {
            while let Some(task) = receiver.next().await {
                let task: Box<dyn FnOnce(&mut State) + Send + 'static> = task;
                task(&mut state);
            }
        });
        Self {
            sender,
        }
    }

    pub fn schedule_direct<F>(&mut self, task: F)
    where
        F: FnOnce(&mut State) + Send + 'static,
    {
        executor::block_on(async {
            self.sender.send_direct(Box::new(task)).await.unwrap_or(());
        });
    }

    pub fn schedule<F>(&mut self, task: F)
    where
        F: FnOnce(&mut State) + Send + 'static,
    {
        use futures::SinkExt;
        executor::block_on(async {
            self.sender.send(Box::new(task)).await.unwrap_or(());
        });
    }

    pub fn schedule_delayed<F>(&mut self, delay: time::Duration, task: F)
    where
        F: FnOnce(&mut State) + Send + 'static,
    {
        executor::block_on(async {
            self.sender.send_delayed(delay, Box::new(task)).await.unwrap_or(())
        });
    }
}