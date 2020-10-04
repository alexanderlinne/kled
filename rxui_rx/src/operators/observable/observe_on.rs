use crate::core;
use crate::sync::mpsc;
use crate::sync::{Arc, Mutex};
use std::cell::UnsafeCell;
use std::marker::PhantomData;

#[derive(new, reactive_operator)]
pub struct ObservableObserveOn<Observable, Scheduler>
where
    Observable: core::SharedObservable,
    Observable::Cancellable: Send,
    Observable::Item: Send,
    Observable::Error: Send,
    Scheduler: core::Scheduler + Send + 'static,
{
    #[upstream()]
    observable: Observable,
    scheduler: Scheduler,
}

struct ObserveOnObserver<Observer, Scheduler, Cancellable, Item, Error> {
    task: Arc<ObserveOnTaskWrapper<Observer, Cancellable, Item, Error>>,
    sender: mpsc::Sender<Item>,
    scheduler: Scheduler,
    phantom: PhantomData<Cancellable>,
}

impl<Observer, Scheduler, Cancellable, Item, Error>
    ObserveOnObserver<Observer, Scheduler, Cancellable, Item, Error>
where
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    fn new(observer: Observer, scheduler: Scheduler) -> Self {
        let (sender, receiver) = mpsc::unbounded();
        ObserveOnObserver {
            task: Arc::new(ObserveOnTaskWrapper::new(receiver, observer)),
            sender,
            scheduler,
            phantom: PhantomData,
        }
    }

    fn schedule<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Data<Error>),
    {
        let last_pending_count = unsafe {
            let mut data = (*self.task.inner.get()).data.lock();
            let last_pending_count = data.pending_count;
            f(&mut data);
            last_pending_count
        };
        if last_pending_count == 0 {
            let task = self.task.clone();
            self.scheduler.schedule(move || unsafe {
                (*task.inner.get()).drain();
            });
        }
    }
}

impl<Cancellable, Item, Error, Observer, Scheduler> core::Observer<Cancellable, Item, Error>
    for ObserveOnObserver<Observer, Scheduler, Cancellable, Item, Error>
where
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
    Scheduler: core::Scheduler + Send + 'static,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        unsafe {
            (*self.task.inner.get()).observer.on_subscribe(cancellable);
        }
    }

    fn on_next(&mut self, item: Item) {
        self.sender.send(item).unwrap();
        self.schedule(|data| data.pending_count += 1);
    }

    fn on_error(&mut self, error: Error) {
        self.schedule(move |data| {
            data.error = Some(error);
            data.done = true;
        });
    }

    fn on_completed(&mut self) {
        self.schedule(|data| data.done = true);
    }
}

struct ObserveOnTaskWrapper<Observer, Cancellable, Item, Error> {
    inner: UnsafeCell<ObserveOnTask<Observer, Cancellable, Item, Error>>,
}

impl<Observer, Cancellable, Item, Error> ObserveOnTaskWrapper<Observer, Cancellable, Item, Error> {
    pub fn new(receiver: mpsc::Receiver<Item>, observer: Observer) -> Self {
        Self {
            inner: UnsafeCell::new(ObserveOnTask::new(receiver, observer)),
        }
    }
}

unsafe impl<Observer, Cancellable, Item, Error> Send
    for ObserveOnTaskWrapper<Observer, Cancellable, Item, Error>
{
}
unsafe impl<Observer, Cancellable, Item, Error> Sync
    for ObserveOnTaskWrapper<Observer, Cancellable, Item, Error>
{
}

struct ObserveOnTask<Observer, Cancellable, Item, Error> {
    receiver: mpsc::Receiver<Item>,
    data: Mutex<Data<Error>>,
    observer: Observer,
    phantom: PhantomData<Cancellable>,
}

impl<Observer, Cancellable, Item, Error> ObserveOnTask<Observer, Cancellable, Item, Error> {
    pub fn new(receiver: mpsc::Receiver<Item>, observer: Observer) -> Self {
        Self {
            receiver,
            data: Mutex::new(Data::default()),
            observer,
            phantom: PhantomData,
        }
    }
}

struct Data<Error> {
    pending_count: usize,
    done: bool,
    error: Option<Error>,
}

impl<Error> Default for Data<Error> {
    fn default() -> Self {
        Self {
            pending_count: 0,
            done: false,
            error: None,
        }
    }
}

impl<Observer, Cancellable, Item, Error> ObserveOnTask<Observer, Cancellable, Item, Error>
where
    Observer: core::Observer<Cancellable, Item, Error>,
{
    pub fn drain(&mut self) {
        let mut expected_count: usize = 0;
        loop {
            for _ in 0..expected_count {
                let mut data = self.data.lock();
                if data.done && data.error.is_some() {
                    self.observer.on_error(data.error.take().unwrap());
                    return;
                }
                drop(data);

                let item = self.receiver.recv().unwrap();
                self.observer.on_next(item);
            }

            let mut data = self.data.lock();
            data.pending_count -= expected_count;
            expected_count = data.pending_count;
            if expected_count == 0 {
                if data.done {
                    if data.error.is_some() {
                        self.observer.on_error(data.error.take().unwrap());
                    } else {
                        self.observer.on_completed();
                    }
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::observable::shared::*;
    use crate::observer::shared::*;
    use crate::prelude::*;
    use crate::scheduler;

    #[test]
    fn observe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_shared_observable()
            .observe_on(scheduler.clone())
            .subscribe(test_observer.clone());
        scheduler.join();
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn observe_on_shared() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_shared_observable()
            .observe_on(scheduler.clone())
            .subscribe(test_observer.clone());
        scheduler.join();
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn observe_on_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        let test_observable = TestObservable::default().annotate_item_type(());
        test_observable
            .clone()
            .observe_on(scheduler.clone())
            .subscribe(test_observer.clone());
        test_observable.emit_error(());
        scheduler.join();
        assert_eq!(test_observer.status(), ObserverStatus::Error);
        assert_eq!(test_observer.error(), Some(()));
    }
}
