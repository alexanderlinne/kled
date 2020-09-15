mod create;

pub use create::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BackpressureStrategy {
    Missing,
    Error,
    Drop,
    Latest,
    Buffer,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BufferStrategy {
    Error,
    DropLatest,
    DropOldest,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Error<UpstreamError> {
    Upstream(UpstreamError),
    BackpressureError,
}

impl<UpstreamError> Error<UpstreamError> {
    pub fn is_backpressure_error(&self) -> bool {
        matches! {self, Self::BackpressureError}
    }
}
