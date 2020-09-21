use crate::core;
use crate::flow;
use crate::subscription::local::*;

impl<'o, Subscriber, Item, Error> core::IntoFlowEmitter<'o, Item, Error> for Subscriber
where
    Subscriber: core::Subscriber<BoolSubscription, Item, Error> + 'o,
    Item: 'o,
    Error: 'o,
{
    type Emitter = Box<dyn core::FlowEmitter<Item, Error> + 'o>;

    fn into_emitter(self, strategy: flow::BackpressureStrategy) -> Self::Emitter {
        match strategy {
            flow::BackpressureStrategy::Missing => Box::new(super::flow::MissingEmitter::new(self)),
            flow::BackpressureStrategy::Drop => Box::new(super::flow::DropEmitter::new(self)),
            _ => unimplemented! {},
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::util::local::*;

    #[test]
    fn missing() {
        let test_subscriber = TestSubscriber::new(1);
        vec![0, 1, 2]
            .into_flow(flow::BackpressureStrategy::Missing)
            .subscribe(test_subscriber.clone());
        assert_eq!(test_subscriber.status(), ObserverStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2]);
    }

    #[test]
    fn missing_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::new(flow::BackpressureStrategy::Missing);
        test_flow.clone().subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }

    #[test]
    fn drop() {
        let test_subscriber = TestSubscriber::new(1);
        vec![0, 1, 2]
            .into_flow(flow::BackpressureStrategy::Drop)
            .subscribe(test_subscriber.clone());
        assert_eq!(test_subscriber.status(), ObserverStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
    }

    #[test]
    fn drop_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::new(flow::BackpressureStrategy::Drop);
        test_flow.clone().subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.items(), vec![]);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }
}