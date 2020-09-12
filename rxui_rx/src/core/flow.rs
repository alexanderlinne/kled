pub trait Flow {
    type Item;
    type Error;
}

pub trait IntoFlow {
    type FlowType: Flow;

    fn into_flow(self) -> Self::FlowType;
}

pub enum BackpressureStrategy {
    Error,
    Drop,
    Latest,
    Buffer,
}

pub enum BufferStrategy {
    Error,
    DropLatest,
    DropOldest,
}

pub enum FlowError<UpstreamError> {
    Upstream(UpstreamError),
    BackpressureError,
}
