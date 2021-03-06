#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Error<UpstreamError> {
    Upstream(UpstreamError),
    MissingBackpressure,
}

impl<UpstreamError> Error<UpstreamError> {
    pub fn is_backpressure_error(&self) -> bool {
        matches! {self, Self::MissingBackpressure}
    }
}
