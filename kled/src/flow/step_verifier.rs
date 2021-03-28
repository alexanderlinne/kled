use crate::{core, flow};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

enum Action {
    Request(usize),
    Then(Box<dyn FnOnce() + Send + 'static>),
}

struct Actions {
    actions: Vec<Action>,
}

impl Default for Actions {
    fn default() -> Self {
        Self {
            actions: Default::default(),
        }
    }
}

impl Actions {
    pub fn add(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub async fn execute<Subscription>(self, subscription: &Subscription)
    where
        Subscription: core::Subscription + Send + Sync,
    {
        for action in self.actions {
            match action {
                Action::Request(count) => subscription.request(count).await,
                Action::Then(f) => f(),
            }
        }
    }
}

#[derive(Debug)]
enum Signal<Item, Error> {
    Subscribe,
    Next(Item),
    Error(flow::Error<Error>),
    Completed,
}

impl<Item, Error> Display for Signal<Item, Error>
where
    Item: Debug,
    Error: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Subscribe => f.write_str("on_subscribe"),
            Self::Next(item) => f.write_fmt(format_args!("on_next with item {:?}", item)),
            Self::Error(error) => f.write_fmt(format_args!("on_error with error {:?}", error)),
            Self::Completed => f.write_str("on_completed"),
        }
    }
}

struct Step<Item, Error> {
    signal: Signal<Item, Error>,
    actions: Actions,
}

impl<Item, Error> From<Signal<Item, Error>> for Step<Item, Error> {
    fn from(signal: Signal<Item, Error>) -> Step<Item, Error> {
        Step {
            signal,
            actions: Default::default(),
        }
    }
}

struct Steps<Subscription, Item, Error> {
    steps: VecDeque<Step<Item, Error>>,
    phantom: PhantomData<Subscription>,
}

impl<Subscription, Item, Error> Steps<Subscription, Item, Error> {
    pub fn add(&mut self, step: Step<Item, Error>) {
        self.steps.push_back(step);
    }

    pub fn add_action(&mut self, action: Action) {
        self.steps.back_mut().unwrap().actions.add(action);
    }

    pub fn pop(&mut self) -> Option<Step<Item, Error>> {
        self.steps.pop_front()
    }
}

#[must_use]
pub struct StepVerifier<Flow, Subscription, Item, Error> {
    flow: Flow,
    steps: Steps<Subscription, Item, Error>,
}

impl<Flow, Subscription, Item, Error> StepVerifier<Flow, Subscription, Item, Error>
where
    Flow: core::Flow<Subscription, Item, Error>,
    Subscription: core::Subscription + Send + Sync + 'static,
    Item: Debug + PartialEq + Send + 'static,
    Error: Debug + PartialEq + Send + 'static,
{
    pub fn create(flow: Flow) -> FirstStepBuilder<Flow, Subscription, Item, Error> {
        FirstStepBuilder::new(flow)
    }

    pub async fn verify(self) {
        self.flow
            .subscribe(VerificationSubscriber::new(self.steps))
            .await;
    }
}

#[must_use]
pub struct FirstStepBuilder<Flow, Subscription, Item, Error> {
    flow: Flow,
    steps: Steps<Subscription, Item, Error>,
}

impl<Flow, Subscription, Item, Error> FirstStepBuilder<Flow, Subscription, Item, Error>
where
    Item: Debug + PartialEq,
    Error: Debug + PartialEq,
{
    fn new(flow: Flow) -> Self {
        Self {
            flow,
            steps: Steps {
                steps: VecDeque::new(),
                phantom: PhantomData,
            },
        }
    }

    pub fn expect_subscription(mut self) -> StepBuilder<Flow, Subscription, Item, Error> {
        self.steps.add(Signal::Subscribe.into());
        StepBuilder::new(self.flow, self.steps)
    }
}

#[must_use]
pub struct StepBuilder<Flow, Subscription, Item, Error> {
    flow: Flow,
    steps: Steps<Subscription, Item, Error>,
}

impl<Flow, Subscription, Item, Error> StepBuilder<Flow, Subscription, Item, Error>
where
    Item: Debug + PartialEq,
    Error: Debug + PartialEq,
{
    fn new(flow: Flow, steps: Steps<Subscription, Item, Error>) -> Self {
        Self { flow, steps }
    }

    pub fn expect_next(mut self, item: Item) -> Self {
        self.steps.add(Signal::Next(item).into());
        self
    }

    pub fn expect_all_of<Iter>(mut self, iter: Iter) -> Self
    where
        Iter: IntoIterator<Item = Item>,
    {
        for item in iter.into_iter() {
            self.steps.add(Signal::Next(item).into());
        }
        self
    }

    pub fn expect_error(
        mut self,
        error: flow::Error<Error>,
    ) -> StepVerifier<Flow, Subscription, Item, Error> {
        self.steps.add(Signal::Error(error).into());
        StepVerifier {
            flow: self.flow,
            steps: self.steps,
        }
    }

    pub fn expect_completed(mut self) -> StepVerifier<Flow, Subscription, Item, Error> {
        self.steps.add(Signal::Completed.into());
        StepVerifier {
            flow: self.flow,
            steps: self.steps,
        }
    }

    pub fn and_request(mut self, count: usize) -> Self {
        self.steps.add_action(Action::Request(count));
        self
    }

    pub fn then<F>(mut self, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        self.steps.add_action(Action::Then(Box::new(f)));
        self
    }
}

struct VerificationSubscriber<Subscription, Item, Error> {
    subscription: Option<Subscription>,
    steps: Steps<Subscription, Item, Error>,
}

impl<Subscription, Item, Error> VerificationSubscriber<Subscription, Item, Error> {
    pub fn new(steps: Steps<Subscription, Item, Error>) -> Self {
        Self {
            subscription: None,
            steps,
        }
    }
}

#[async_trait]
impl<Subscription, Item, Error> core::Subscriber<Subscription, Item, Error>
    for VerificationSubscriber<Subscription, Item, Error>
where
    Subscription: core::Subscription + Send + Sync,
    Item: Debug + PartialEq + Send,
    Error: Debug + PartialEq + Send,
{
    async fn on_subscribe(&mut self, subscription: Subscription) {
        self.subscription = Some(subscription);
        let step = self
            .steps
            .pop()
            .expect("StepVerifier: expected no signal, but received on_subscribe");
        if !matches!(step.signal, Signal::Subscribe) {
            panic! {"StepVerifier: expected {}, but received on_subscribe", step.signal};
        }
        step.actions
            .execute(self.subscription.as_ref().unwrap())
            .await;
    }

    async fn on_next(&mut self, item: Item) {
        let step = self.steps.pop().unwrap_or_else(|| {
            panic!(
                "StepVerifier: expected no signal, but received on_next with item {:?}",
                item
            )
        });
        if let Signal::Next(expcted_item) = step.signal {
            assert!(
                item == expcted_item,
                "StepVerifier: on_next: expected {:?}, but received {:?}",
                expcted_item,
                item
            );
        } else {
            panic! {"StepVerifier: expected {}, but received on_next with item {:?}", step.signal, item};
        }
        step.actions
            .execute(self.subscription.as_ref().unwrap())
            .await;
    }

    async fn on_error(&mut self, error: flow::Error<Error>) {
        let step = self.steps.pop().unwrap_or_else(|| {
            panic!(
                "StepVerifier: expected no signal, but received on_error with item {:?}",
                error
            )
        });
        if let Signal::Error(expcted_error) = step.signal {
            assert!(
                error == expcted_error,
                "StepVerifier: on_error: expected {:?}, but received {:?}",
                expcted_error,
                error
            );
        } else {
            panic! {"StepVerifier: expected {}, but received on_error with error {:?}", step.signal, error};
        }
        step.actions
            .execute(self.subscription.as_ref().unwrap())
            .await;
    }

    async fn on_completed(&mut self) {
        let step = self
            .steps
            .pop()
            .expect("StepVerifier: expected no signal, but received on_completed");
        if !matches!(step.signal, Signal::Completed) {
            panic! {"StepVerifier: expected {}, but received on_completed", step.signal};
        }
        step.actions
            .execute(self.subscription.as_ref().unwrap())
            .await;
    }
}
