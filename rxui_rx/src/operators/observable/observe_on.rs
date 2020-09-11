use crate::core;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub struct ObservableObserveOn<Observable, Worker> {
    observable: Observable,
    worker: Worker,
}

impl<Observable, Worker> ObservableObserveOn<Observable, Worker>
where
    Worker: core::Worker + Send + 'static,
{
    pub fn new(observable: Observable, worker: Worker) -> Self {
        ObservableObserveOn { observable, worker }
    }
}

impl<Observable, Worker> core::SharedObservable for ObservableObserveOn<Observable, Worker>
where
    Observable: core::SharedObservable + 'static,
    <Observable as core::SharedObservable>::Cancellable: Send,
    <Observable as core::Observable>::Item: Send,
    <Observable as core::Observable>::Error: Send,
    Worker: core::Worker + Send + 'static,
{
    type Cancellable = Observable::Cancellable;

    fn actual_subscribe<Observer>(self, observer: Observer)
    where
        Observer: core::Observer<Self::Cancellable, Self::Item, Self::Error> + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        self.observable.actual_subscribe(ObserveOnObserver {
            task: Arc::new(ObserveOnTaskWrapper {
                inner: UnsafeCell::new(ObserveOnTask {
                    receiver,
                    data: Mutex::new(Data {
                        pending_count: 0,
                        done: false,
                        error: None,
                    }),
                    observer,
                    phantom: PhantomData,
                }),
            }),
            sender,
            worker: self.worker.clone(),
            phantom: PhantomData,
        });
    }
}

impl<Observable, Worker> core::Observable for ObservableObserveOn<Observable, Worker>
where
    Observable: core::Observable,
{
    type Item = Observable::Item;
    type Error = Observable::Error;
}

struct ObserveOnObserver<Observer, Worker, Cancellable, Item, Error> {
    task: Arc<ObserveOnTaskWrapper<Observer, Cancellable, Item, Error>>,
    sender: mpsc::Sender<Item>,
    worker: Worker,
    phantom: PhantomData<Cancellable>,
}

impl<Observer, Worker, Cancellable, Item, Error>
    ObserveOnObserver<Observer, Worker, Cancellable, Item, Error>
where
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
    Worker: core::Worker + Send + 'static,
{
    fn schedule<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Data<Error>),
    {
        let last_pending_count = unsafe {
            let mut data = (*self.task.inner.get()).data.lock().unwrap();
            let last_pending_count = data.pending_count;
            f(&mut data);
            last_pending_count
        };
        if last_pending_count == 0 {
            let task = self.task.clone();
            self.worker.schedule(move || unsafe {
                (*task.inner.get()).drain();
            });
        }
    }
}

impl<Cancellable, Item, Error, Observer, Worker> core::Observer<Cancellable, Item, Error>
    for ObserveOnObserver<Observer, Worker, Cancellable, Item, Error>
where
    Cancellable: Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
    Observer: core::Observer<Cancellable, Item, Error> + Send + 'static,
    Worker: core::Worker + Send + 'static,
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

struct Data<Error> {
    pending_count: usize,
    done: bool,
    error: Option<Error>,
}

impl<Observer, Cancellable, Item, Error> ObserveOnTask<Observer, Cancellable, Item, Error>
where
    Observer: core::Observer<Cancellable, Item, Error>,
{
    pub fn drain(&mut self) {
        let mut expected_count: usize = 1;
        loop {
            for _ in 0..expected_count {
                let mut data = self.data.lock().unwrap();
                if data.done && data.error.is_some() {
                    self.observer.on_error(data.error.take().unwrap());
                    return;
                }
                drop(data);

                let item = self.receiver.recv().unwrap();
                self.observer.on_next(item);
            }

            let mut data = self.data.lock().unwrap();
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
    use crate::prelude::*;
    use crate::scheduler;
    use crate::util::shared::*;

    #[test]
    fn observe_on() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_observer = TestObserver::default();
        vec![0, 1, 2, 3]
            .into_observable()
            .observe_on(&scheduler)
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
            .observe_on(&scheduler)
            .subscribe(test_observer.clone());
        scheduler.join();
        assert_eq!(test_observer.status(), ObserverStatus::Completed);
        assert_eq!(test_observer.items(), vec![0, 1, 2, 3]);
    }
}
