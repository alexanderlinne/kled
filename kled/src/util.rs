#[derive(Copy, Clone)]
pub enum Infallible {}

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

macro_rules! reexport_all {
    ($(mod $idents:ident;)*) => {
        $(
            mod $idents;
            pub use $idents::*;
        )*
    };
}
