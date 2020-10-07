pub mod sync;

#[cfg(test)]
pub mod thread;
#[cfg(not(test))]
pub use std::thread;

#[cfg(test)]
pub mod time;
#[cfg(not(test))]
pub use std::time;
