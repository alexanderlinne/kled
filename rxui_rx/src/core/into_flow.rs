use crate::core;

pub trait IntoFlow {
    type Flow: core::Flow;

    fn into_flow(self) -> Self::Flow;
}

pub trait IntoSharedFlow {
    type Flow: core::SharedFlow;

    fn into_shared_flow(self) -> core::Shared<Self::Flow>;
}

impl<T> IntoSharedFlow for T
where
    T: IntoFlow,
    T::Flow: core::SharedFlow,
{
    type Flow = T::Flow;

    fn into_shared_flow(self) -> core::shared::Shared<Self::Flow> {
        use crate::core::SharedFlow;
        self.into_flow().into_shared()
    }
}
