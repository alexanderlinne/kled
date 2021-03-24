#[derive(Copy, Clone)]
pub enum Infallible {}

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
