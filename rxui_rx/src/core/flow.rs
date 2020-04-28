pub trait Flow {
    type Item;
    type Error;
}

pub trait IntoFlow {
    type FlowType: Flow;

    fn into_flow(self) -> Self::FlowType;
}

pub trait FlowSubscription {
    fn cancel(self);

    fn request(&self, count: usize);
}
