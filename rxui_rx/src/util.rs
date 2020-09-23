use crate::core;
use crate::marker;

pub trait Sealed {}

impl<'o, Observable> Sealed for Observable where Observable: core::LocalObservable<'o> + 'o {}

impl<Observable> Sealed for marker::Shared<Observable> where
    Observable: core::SharedObservable + Send + 'static
{
}

impl<'o, Flow> Sealed for marker::Flow<Flow> where Flow: core::LocalFlow<'o> + 'o {}

impl<Flow> Sealed for marker::Shared<marker::Flow<Flow>> where
    Flow: core::SharedFlow + Send + 'static
{
}

pub trait Inconstructible: Sealed {}

#[derive(Copy, Clone)]
pub enum Infallible {}

impl Sealed for Infallible {}

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
