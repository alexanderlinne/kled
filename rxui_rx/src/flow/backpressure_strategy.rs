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
