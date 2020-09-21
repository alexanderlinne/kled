use crate::core;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct LambdaSubscriptionStub<'o> {
    data: Rc<RefCell<Data<'o>>>,
}

impl<'o> LambdaSubscriptionStub<'o> {
    pub fn new<RequestFn>(request_fn: RequestFn) -> Self
    where
        RequestFn: Fn(usize) + 'o,
    {
        Self {
            data: Rc::new(RefCell::new(Data {
                cancelled: false,
                request_fn: Box::new(request_fn),
            })),
        }
    }
}

impl<'o> LambdaSubscriptionStub<'o> {
    pub fn subscription(&self) -> LambdaSubscription<'o> {
        LambdaSubscription {
            data: self.data.clone(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        (*self.data.borrow()).cancelled
    }
}

#[derive(Clone)]
pub struct LambdaSubscription<'o> {
    data: Rc<RefCell<Data<'o>>>,
}

impl<'o> core::Subscription for LambdaSubscription<'o> {
    fn cancel(&self) {
        (*self.data.borrow_mut()).cancelled = true;
    }

    fn is_cancelled(&self) -> bool {
        (*self.data.borrow()).cancelled
    }

    fn request(&self, count: usize) {
        ((*self.data.borrow()).request_fn)(count);
    }
}

struct Data<'o> {
    cancelled: bool,
    request_fn: Box<dyn Fn(usize) + 'o>,
}
