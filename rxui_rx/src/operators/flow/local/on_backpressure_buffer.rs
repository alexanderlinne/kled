use crate::core;
use crate::flow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

#[derive(new, reactive_operator)]
pub struct FlowOnBackpressureBuffer<'o, Flow>
where
    Flow: core::LocalFlow<'o>,
{
    #[upstream(
        subscription = "OnBackpressureBufferSubscription<'o, Flow::Subscription, Flow::Item, Flow::Error>"
    )]
    flow: Flow,
    buffer_strategy: flow::BufferStrategy,
    buffer_capacity: usize,
    #[reactive_operator(ignore)]
    phantom: PhantomData<&'o Self>,
}

pub struct OnBackpressureBufferSubscriber<'o, Subscription, Item, Error> {
    data: Rc<Data<'o, Subscription, Item, Error>>,
    buffer_strategy: flow::BufferStrategy,
}

type SubscriberTy<'o, Subscription, Item, Error> = Box<
    dyn core::Subscriber<
            OnBackpressureBufferSubscription<'o, Subscription, Item, Error>,
            Item,
            Error,
        > + 'o,
>;

pub struct Data<'o, Subscription, Item, Error> {
    subscriber: RefCell<Option<SubscriberTy<'o, Subscription, Item, Error>>>,
    requested: RefCell<usize>,
    queue: RefCell<VecDeque<Item>>,
}

impl<'o, Subscription, Item, Error> OnBackpressureBufferSubscriber<'o, Subscription, Item, Error>
where
    Item: 'o,
{
    pub fn new<Subscriber>(
        subscriber: Subscriber,
        buffer_strategy: flow::BufferStrategy,
        buffer_capacity: usize,
    ) -> Self
    where
        Subscriber: core::Subscriber<
                OnBackpressureBufferSubscription<'o, Subscription, Item, Error>,
                Item,
                Error,
            > + 'o,
    {
        let data = Rc::new(Data {
            subscriber: RefCell::new(Some(Box::new(subscriber))),
            requested: RefCell::new(0),
            queue: RefCell::new(VecDeque::with_capacity(buffer_capacity)),
        });
        Self {
            data,
            buffer_strategy,
        }
    }

    fn add_to_queue(&self, item: Item) {
        let mut queue = self.data.queue.borrow_mut();
        if queue.len() < queue.capacity() {
            queue.push_back(item);
        } else {
            use flow::BufferStrategy::*;
            match self.buffer_strategy {
                Error => {
                    if let Some(mut subscriber) = self.data.subscriber.borrow_mut().take() {
                        subscriber.on_error(flow::Error::MissingBackpressure)
                    };
                }
                DropOldest => {
                    queue.pop_front();
                    queue.push_back(item);
                }
                DropLatest => {}
            }
        }
    }
}

fn drain<'o, Subscription, Item, Error>(
    data: &Rc<Data<'o, Subscription, Item, Error>>,
    subscriber: &mut SubscriberTy<'o, Subscription, Item, Error>,
    mut requested: usize,
) {
    let mut queue = data.queue.borrow_mut();
    let mut emitted = 0;
    while !queue.is_empty() && emitted < requested {
        subscriber.on_next(queue.pop_front().unwrap());
        emitted += 1;
        // If the loop would finish, update the count of requested items as
        // on_next may have called request
        if emitted == requested {
            requested = *data.requested.borrow();
        }
    }
    *data.requested.borrow_mut() = requested - emitted;
}

impl<'o, Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for OnBackpressureBufferSubscriber<'o, Subscription, Item, Error>
where
    Item: 'o,
{
    fn on_subscribe(&mut self, subscription: Subscription) {
        let data = Rc::downgrade(&self.data);
        if let Some(subscriber) = self.data.subscriber.borrow_mut().as_mut() {
            subscriber.on_subscribe(OnBackpressureBufferSubscription::new(subscription, data))
        };
    }

    fn on_next(&mut self, item: Item) {
        let requested = *self.data.requested.borrow();
        self.add_to_queue(item);
        if requested > 0 {
            if let Some(ref mut subscriber) = *self.data.subscriber.borrow_mut() {
                drain(&self.data, subscriber, requested);
            }
        }
    }

    fn on_error(&mut self, error: flow::Error<Error>) {
        if let Some(subscriber) = self.data.subscriber.borrow_mut().as_mut() {
            subscriber.on_error(error)
        };
    }

    fn on_completed(&mut self) {
        if let Some(subscriber) = self.data.subscriber.borrow_mut().as_mut() {
            subscriber.on_completed()
        };
    }
}

#[derive(new)]
pub struct OnBackpressureBufferSubscription<'o, Upstream, Item, Error> {
    upstream: Upstream,
    data: Weak<Data<'o, Upstream, Item, Error>>,
}

impl<'o, Upstream, Item, Error> core::Subscription
    for OnBackpressureBufferSubscription<'o, Upstream, Item, Error>
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

        let mut requested_ref = data.requested.borrow_mut();
        *requested_ref += count;
        let requested = *requested_ref;
        drop(requested_ref);

        if requested > 0 {
            // This prevents more than one reentrant call of request is the
            // subscriber is borrowed mutably either here or in on_next
            if let Ok(mut subscriber) = data.subscriber.try_borrow_mut() {
                if let Some(mut subscriber) = (&mut *subscriber).as_mut() {
                    drain(&data, &mut subscriber, requested)
                };
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
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::Error, 5)
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
        assert_eq!(test_subscriber.items(), vec![0, 1, 2]);
    }

    #[test]
    fn upstream_error() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default();
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::Error, 1)
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
    }

    #[test]
    fn error_strategy() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::Error, 1)
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit(1);
        test_subscriber.request_direct(1);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
    }

    #[test]
    fn drop_oldest_strategy() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::DropOldest, 1)
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit(1);
        test_subscriber.request_direct(1);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![1]);
    }

    #[test]
    fn drop_latest_strategy() {
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::default().annotate_error_type(());
        test_flow
            .clone()
            .on_backpressure_buffer_with_capacity(flow::BufferStrategy::DropLatest, 1)
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit(1);
        test_subscriber.request_direct(1);
        test_flow.emit_completed();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
    }
}
