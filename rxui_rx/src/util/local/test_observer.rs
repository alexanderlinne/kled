use crate::core;
use std::cell::RefCell;
use std::rc::Rc;

pub struct TestObserver<Cancellable, Item, Error> {
    data: Rc<RefCell<Data<Cancellable, Item, Error>>>,
}

struct Data<Cancellable, Item, Error> {
    cancellable: Option<Cancellable>,
    items: Vec<Item>,
    error: Option<Error>,
    is_completed: bool,
    is_cancelled: bool,
}

impl<Cancellable, Item, Error> Default for TestObserver<Cancellable, Item, Error> {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new(Data {
                cancellable: None,
                items: vec![],
                error: None,
                is_completed: false,
                is_cancelled: false,
            })),
        }
    }
}

impl<Cancellable, Item, Error> Clone for TestObserver<Cancellable, Item, Error> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ObserverStatus {
    Unsubscribed,
    Subscribed,
    Error,
    Completed,
    Cancelled,
}

impl<Cancellable, Item, Error> TestObserver<Cancellable, Item, Error>
where
    Cancellable: core::Cancellable,
{
    pub fn status(&self) -> ObserverStatus {
        if self.data.borrow().is_cancelled {
            return ObserverStatus::Cancelled;
        }

        if !self.is_subscribed() {
            return ObserverStatus::Unsubscribed;
        }

        if self.has_error() {
            return ObserverStatus::Error;
        }

        if self.is_completed() {
            return ObserverStatus::Completed;
        }

        return ObserverStatus::Subscribed;
    }

    pub fn is_subscribed(&self) -> bool {
        self.data.borrow().cancellable.is_some()
    }

    pub fn cancel_cancellable(&mut self) {
        assert!(self.is_subscribed());
        let mut data = self.data.borrow_mut();
        data.cancellable.take().unwrap().cancel();
        data.is_cancelled = true;
    }

    pub fn has_error(&self) -> bool {
        self.data.borrow().error.is_some()
    }

    pub fn is_completed(&self) -> bool {
        self.data.borrow().is_completed
    }
}

impl<Cancellable, Item, Error> TestObserver<Cancellable, Item, Error>
where
    Item: Clone,
    Error: Clone,
{
    pub fn items(&self) -> Vec<Item> {
        self.data.borrow().items.clone()
    }

    pub fn error(&self) -> Option<Error> {
        self.data.borrow().error.clone()
    }
}

impl<Cancellable, Item, Error> core::Observer<Cancellable, Item, Error>
    for TestObserver<Cancellable, Item, Error>
where
    Cancellable: core::Cancellable,
{
    fn on_subscribe(&mut self, cancellable: Cancellable) {
        assert_eq!(self.status(), ObserverStatus::Unsubscribed);
        self.data.borrow_mut().cancellable = Some(cancellable);
    }
    fn on_next(&mut self, item: Item) {
        assert_eq!(self.status(), ObserverStatus::Subscribed);
        self.data.borrow_mut().items.push(item);
    }
    fn on_error(&mut self, error: Error) {
        assert_eq!(self.status(), ObserverStatus::Subscribed);
        self.data.borrow_mut().error = Some(error)
    }
    fn on_completed(&mut self) {
        assert_eq!(self.status(), ObserverStatus::Subscribed);
        self.data.borrow_mut().is_completed = true;
    }
}
