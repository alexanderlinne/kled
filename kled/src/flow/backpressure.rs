#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BufferStrategy {
    Error,
    DropLatest,
    DropOldest,
}

static BUFFER_CAPACITY: usize = 128;

pub fn default_buffer_capacity() -> usize {
    BUFFER_CAPACITY
}
