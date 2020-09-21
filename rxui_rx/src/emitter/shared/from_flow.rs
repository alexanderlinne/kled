use crate::core;
use crate::flow;
use crate::subscription::shared::*;

impl<Subscriber, Item, Error> core::IntoSharedFlowEmitter<Item, Error> for Subscriber
where
    Subscriber: core::Subscriber<BoolSubscription, Item, Error> + Send + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
    type Emitter = Box<dyn core::FlowEmitter<Item, Error> + Send + 'static>;

    fn into_shared_emitter(self, strategy: flow::BackpressureStrategy) -> Self::Emitter {
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
    use crate::util::shared::*;

    #[test]
    fn missing() {
        let test_subscriber = TestSubscriber::new(1);
        let scheduler = scheduler::NewThreadScheduler::default();
        vec![0, 1, 2]
            .into_flow(flow::BackpressureStrategy::Missing)
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), ObserverStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0, 1, 2]);
    }

    #[test]
    fn drop() {
        let test_subscriber = TestSubscriber::new(1);
        let scheduler = scheduler::NewThreadScheduler::default();
        vec![0, 1, 2]
            .into_flow(flow::BackpressureStrategy::Drop)
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        scheduler.join();
        assert_eq!(test_subscriber.status(), ObserverStatus::Completed);
        assert_eq!(test_subscriber.items(), vec![0]);
    }

    #[test]
    fn drop_error() {
        let scheduler = scheduler::ThreadPoolScheduler::default();
        let test_subscriber = TestSubscriber::default();
        let test_flow = TestFlow::new(flow::BackpressureStrategy::Drop);
        test_flow
            .clone()
            .observe_on(scheduler.clone())
            .subscribe(test_subscriber.clone());
        test_flow.emit(0);
        test_flow.emit_error(());
        scheduler.join();
        assert_eq!(test_subscriber.status(), SubscriberStatus::Error);
        assert_eq!(test_subscriber.error(), Some(flow::Error::Upstream(())));
    }
}