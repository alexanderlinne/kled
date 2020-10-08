use crate::core;

pub trait ObservableSealed {}

impl<Observable> ObservableSealed for Observable where Observable: core::Observable + Send + 'static {}

pub trait FlowSealed {}

impl<Flow> FlowSealed for Flow where Flow: core::Flow + Send + 'static {}

pub trait Inconstructible: ObservableSealed + FlowSealed {}

#[derive(Copy, Clone)]
pub enum Infallible {}

impl ObservableSealed for Infallible {}
impl FlowSealed for Infallible {}

impl Inconstructible for Infallible {}

pub(crate) fn distribute_value<T, F, Value>(vec: &mut Vec<T>, f: F, value: Value)
where
    F: Fn(&mut T, Value),
    Value: Clone,
{
    match vec.len() {
        0 => (),
        1 => f(&mut vec[0], value),
        len => {
            vec.iter_mut()
                .take(len - 1)
                .for_each(|t| f(t, value.clone()));
            f(&mut vec[len - 1], value);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DownstreamStatus {
    Unsubscribed,
    Subscribed,
    Error,
    Completed,
    Cancelled,
}
