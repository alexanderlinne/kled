use crate::core;
use crate::flow;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

#[derive(new, reactive_operator)]
pub struct FlowOnBackpressureLatest<'o, Flow>
where
    Flow: core::LocalFlow<'o>,
{
    #[upstream(
        subscription = "OnBackpressureLatestSubscription<'o, Flow::Subscription, Flow::Item, Flow::Error>"
    )]
    flow: Flow,
    #[reactive_operator(ignore)]
    phantom: PhantomData<&'o Self>,
}

type BoxedSubscriber<'o, Subscription, Item, Error> = Box<
    dyn core::Subscriber<
            OnBackpressureLatestSubscription<'o, Subscription, Item, Error>,
            Item,
            Error,
        > + 'o,
>;

pub struct OnBackpressureLatestSubscriber<'o, Subscription, Item, Error> {
    subscriber: Rc<RefCell<BoxedSubscriber<'o, Subscription, Item, Error>>>,
    data: Rc<RefCell<Data<Item>>>,
    phantom: PhantomData<(Subscription, Error)>,
}

pub struct Data<Item> {
    requested: usize,
    latest: Option<Item>,
}

impl<'o, Subscription, Item, Error> OnBackpressureLatestSubscriber<'o, Subscription, Item, Error>
where
    Item: 'o,
{
    pub fn new<Subscriber>(subscriber: Subscriber) -> Self
    where
        Subscriber: core::Subscriber<
                OnBackpressureLatestSubscription<'o, Subscription, Item, Error>,
                Item,
                Error,
            > + 'o,
    {
        Self {
            subscriber: Rc::new(RefCell::new(Box::new(subscriber))),
            data: Rc::new(RefCell::new(Data {
                requested: 0,
                latest: None,
            })),
            phantom: PhantomData,
        }
    }
}

impl<'o, Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureLatestSubscriber<'o, Subscription, Item, Error>
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        let subscription = OnBackpressureLatestSubscription::new(
            subscription,
            Rc::downgrade(&self.subscriber),
            Rc::downgrade(&self.data),
        );
        self.subscriber.borrow_mut().on_subscribe(subscription);
    }

    fn on_next(&mut self, item: Item) {
        // data must be borrowed individually here to allow on_next to call
        // request safely (which borrows data mutably)
        if self.data.borrow().requested > 0 {
            self.subscriber.borrow_mut().on_next(item);
            self.data.borrow_mut().requested -= 1;
        } else {
            self.data.borrow_mut().latest = Some(item);
        }
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        self.subscriber.borrow_mut().on_error(error);
    }

    fn on_completed(&mut self) {
        self.subscriber.borrow_mut().on_completed();
    }
}

#[derive(new)]
pub struct OnBackpressureLatestSubscription<'o, Upstream, Item, Error> {
    upstream: Upstream,
    subscriber: Weak<RefCell<BoxedSubscriber<'o, Upstream, Item, Error>>>,
    data: Weak<RefCell<Data<Item>>>,
    phantom: PhantomData<Error>,
}

impl<'o, Upstream, Item, Error> core::Subscription
    for OnBackpressureLatestSubscription<'o, Upstream, Item, Error>
where
    Upstream: core::Subscription,
{
    fn cancel(&self) {
        self.upstream.cancel()
    }

    fn is_cancelled(&self) -> bool {
        self.upstream.is_cancelled()
    }

    fn request(&self, count: usize) {
        let data = match self.data.upgrade() {
            None => return,
            Some(data) => data,
        };

        let mut data_ref = data.borrow_mut();
        data_ref.requested += count;
        let requested = data_ref.requested;
        drop(data_ref);

        if requested > 0 {
            let item = data.borrow_mut().latest.take();
            // this allows for only one further reentrant call of request
            // because another item cannot be produced here in synchronous
            // code (unless the subscriber is the producer, which is not a
            // sensible use case)
            if let Some(item) = item {
                if let Some(subscriber) = self.subscriber.upgrade() {
                    subscriber.borrow_mut().on_next(item)
                };
                data.borrow_mut().requested -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::flow::local::*;
    use crate::prelude::*;
    use crate::subscriber::local::*;

    #[test]
    fn basic() {
        let mut test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_latest()
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit(1);
        test_subscriber.request_direct(1);
        test_flow.emit(2);
        test_subscriber.request_on_next(1);
        test_flow.emit(3);
        test_subscriber.request_direct(1);
        test_flow.emit(4);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![1, 3, 4]);
    }

    #[test]
    fn error_case() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_latest()
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
    }
}
