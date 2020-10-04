use super::task::Task;
use crate::sync::mpsc::*;
use crate::sync::{Arc, Mutex};
use crate::thread;
use crate::time::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub struct DelayedTask {
    task: Task,
    start_time: Instant,
}

impl DelayedTask {
    pub fn new(task: Task) -> Self {
        Self::with_start_time(task, Instant::now())
    }

    pub fn with_start_time(task: Task, start_time: Instant) -> Self {
        Self { task, start_time }
    }

    pub fn with_delay(task: Task, delay: Duration) -> Option<Self> {
        Instant::now()
            .checked_add(delay)
            .map(|start_time| Self::with_start_time(task, start_time))
    }

    pub fn task(self) -> Task {
        self.task
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    #[cfg(test)]
    pub fn dummy() -> Self {
        Self::new(Task::dummy())
    }

    #[cfg(test)]
    pub fn dummy_with_delay(delay: Duration) -> Option<Self> {
        Self::with_delay(Task::dummy(), delay)
    }
}

impl Ord for DelayedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_time.cmp(&other.start_time).reverse()
    }
}

impl PartialOrd for DelayedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start_time
            .partial_cmp(&other.start_time)
            .map(&Ordering::reverse)
    }
}

impl Eq for DelayedTask {}

impl PartialEq for DelayedTask {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time
    }
}

impl From<Task> for DelayedTask {
    fn from(task: Task) -> Self {
        Self::new(task)
    }
}

pub type DelaySender = Sender<DelayedTask>;

pub struct DelayReceiver {
    receiver: Receiver<DelayedTask>,
    priority_queue: Arc<Mutex<BinaryHeap<DelayedTask>>>,
}

impl DelayReceiver {
    pub fn recv(&self) -> Result<Task, crossbeam::RecvError> {
        let mut queue = self.priority_queue.lock();
        self.drain_channel(&mut queue);
        if let Some(task) = Self::try_pop(&mut queue) {
            return Ok(task);
        }
        if queue.is_empty() {
            match self.receiver.recv() {
                Ok(task) => queue.push(task),
                Err(err) => return Err(err),
            }
        }
        loop {
            match Self::try_pop(&mut queue) {
                Some(task) => return Ok(task),
                None => thread::sleep(
                    queue
                        .peek()
                        .unwrap()
                        .start_time()
                        .saturating_duration_since(Instant::now()),
                ),
            }
        }
    }

    #[allow(dead_code)]
    pub fn try_recv(&self) -> Option<Task> {
        let mut queue = self.priority_queue.lock();
        self.drain_channel(&mut queue);
        Self::try_pop(&mut queue)
    }

    fn drain_channel(&self, queue: &mut BinaryHeap<DelayedTask>) {
        self.receiver.try_iter().for_each(|task| {
            queue.push(task);
        });
    }

    fn try_pop(queue: &mut BinaryHeap<DelayedTask>) -> Option<Task> {
        if let Some(task) = queue.peek() {
            if task.start_time() <= Instant::now() {
                queue.pop().map(&Into::into)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl From<Receiver<DelayedTask>> for DelayReceiver {
    fn from(receiver: Receiver<DelayedTask>) -> Self {
        Self {
            receiver,
            priority_queue: Arc::new(Mutex::new(BinaryHeap::default())),
        }
    }
}

pub fn unbounded() -> (DelaySender, DelayReceiver) {
    let (sender, receiver) = crate::sync::mpsc::unbounded();
    (sender, receiver.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_task() {
        ClockStrategy::set(ClockStrategy::Manual);

        let (tx, rx) = unbounded();
        tx.send(DelayedTask::dummy()).unwrap();
        rx.recv().unwrap().execute()
    }

    #[test]
    fn try_direct_task() {
        ClockStrategy::set(ClockStrategy::Manual);

        let (tx, rx) = unbounded();
        tx.send(DelayedTask::dummy()).unwrap();
        rx.try_recv().unwrap().execute()
    }

    #[test]
    fn delayed_task() {
        ClockStrategy::set(ClockStrategy::AutoInc);

        let task = DelayedTask::with_delay(Task::dummy(), Duration::from_nanos(50)).unwrap();
        let (tx, rx) = unbounded();
        tx.send(task).unwrap();
        rx.recv().unwrap().execute();
        assert_eq!(Instant::get(), Duration::from_nanos(50));
    }

    #[test]
    fn try_delayed_task() {
        ClockStrategy::set(ClockStrategy::Manual);

        let task = DelayedTask::dummy_with_delay(Duration::from_nanos(50)).unwrap();
        let (tx, rx) = unbounded();
        tx.send(task).unwrap();
        matches! {rx.try_recv(), None};
        Instant::inc(Duration::from_nanos(50));
        rx.try_recv().unwrap().execute();
    }

    #[test]
    fn add_direct_after_delayed_task() {
        ClockStrategy::set(ClockStrategy::Manual);

        let (tx, rx) = unbounded();
        tx.send(DelayedTask::dummy_with_delay(Duration::from_nanos(50)).unwrap())
            .unwrap();
        tx.send(DelayedTask::dummy_with_delay(Duration::from_nanos(0)).unwrap())
            .unwrap();
        assert_eq!(Instant::get(), Duration::from_nanos(0));
        rx.recv().unwrap().execute();
        matches! {rx.try_recv(), None};
        Instant::inc(Duration::from_nanos(50));
        rx.recv().unwrap().execute();
    }
}
