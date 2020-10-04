#![allow(dead_code)]
use crate::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::cmp;
use std::thread_local;
use std::time;

pub use time::Duration;

thread_local! {
    // The current clock strategy.
    pub(crate) static CLOCK_MODE: RefCell<Option<ClockStrategy>> = RefCell::new(None);

    // The current time in nanoseconds of the manual clock.
    pub(crate) static MANUAL_NANOS: RefCell<Arc<Mutex<Duration>>> = RefCell::new(Arc::new(Mutex::new(Duration::default())));

    // The carrent time in nanoseconds of the auto incrementing clock.
    pub(crate) static AUTO_INC_NANOS: RefCell<Duration> = RefCell::new(Duration::default());
}

// Specifies the underlying implementation used by the mocked clock. By default
// the system clock is used.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ClockStrategy {
    // The test mock uses the system time. (default)
    Sys,
    // The mocked Instant uses a global nanosecond counter that does not
    // increase unless the user manually requests it.
    Manual,
    // The mocked Instant uses a thread-local counters that automatically
    // increment when e.g. the thread calls thread::sleep. When one thread
    // joins another, the clock of the caller is set to the maximum of both
    // clocks.
    AutoInc,
}

impl ClockStrategy {
    // Returns the current clock mode.
    pub fn current() -> Self {
        CLOCK_MODE.with(|v| v.borrow().to_owned().unwrap_or(Self::Sys))
    }

    // Returns the current clock mode or None if it ClockStrategy::set has not
    // been called yet.
    pub(crate) fn current_raw() -> Option<Self> {
        CLOCK_MODE.with(|v| v.borrow().to_owned())
    }

    // Sets the clock mode globally. This may only be called once per test.
    pub fn set(mode: ClockStrategy) {
        CLOCK_MODE.with(|v| {
            let mut clock_mode = v.borrow_mut();
            matches!(*clock_mode, None);
            *clock_mode = Some(mode);
        });
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Instant {
    Sys(time::Instant),
    Mocked(Duration),
}

impl Instant {
    pub fn inc(duration: Duration) {
        match ClockStrategy::current() {
            ClockStrategy::Sys => panic!("Instant is not mocked!"),
            ClockStrategy::Manual => {
                MANUAL_NANOS.with(|v| *v.borrow().lock() += duration);
            }
            ClockStrategy::AutoInc => {
                AUTO_INC_NANOS.with(|v| *v.borrow_mut() += duration);
            }
        }
    }

    pub fn get() -> Duration {
        match ClockStrategy::current() {
            ClockStrategy::Sys => panic!("Instant is not mocked!"),
            ClockStrategy::Manual => MANUAL_NANOS.with(|v| *v.borrow().lock()),
            ClockStrategy::AutoInc => AUTO_INC_NANOS.with(|v| *v.borrow()),
        }
    }

    pub fn now() -> Self {
        match ClockStrategy::current() {
            ClockStrategy::Sys => Self::Sys(time::Instant::now()),
            ClockStrategy::Manual => Self::Mocked(Self::get()),
            ClockStrategy::AutoInc => Self::Mocked(Self::get()),
        }
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        match self {
            Self::Sys(now) => match earlier {
                Self::Sys(earlier) => now.saturating_duration_since(earlier),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
            Self::Mocked(now) => match earlier {
                Self::Mocked(earlier) => now.checked_sub(earlier).unwrap_or_default(),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
        }
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        match self {
            Self::Sys(instant) => instant.checked_add(duration).map(&Self::Sys),
            Self::Mocked(current_time) => current_time.checked_add(duration).map(&Self::Mocked),
        }
    }
}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        println!("time::Instant::cmp");
        match self {
            Self::Sys(lhs) => match rhs {
                Self::Sys(rhs) => lhs.cmp(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
            Self::Mocked(lhs) => match rhs {
                Self::Mocked(rhs) => lhs.cmp(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
        }
    }
}

impl PartialOrd<Instant> for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        match self {
            Self::Sys(lhs) => match rhs {
                Self::Sys(rhs) => lhs.partial_cmp(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
            Self::Mocked(lhs) => match rhs {
                Self::Mocked(rhs) => lhs.partial_cmp(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
        }
    }
}

impl Eq for Instant {}

impl PartialEq<Instant> for Instant {
    fn eq(&self, rhs: &Self) -> bool {
        match self {
            Self::Sys(lhs) => match rhs {
                Self::Sys(rhs) => lhs.eq(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
            Self::Mocked(lhs) => match rhs {
                Self::Mocked(rhs) => lhs.eq(rhs),
                _ => panic!("Found incompatible instants unexpectedly!"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sync::{Arc, Barrier};
    use crate::thread;
    use crate::time::*;

    #[test]
    fn manual_transfers_on_thread_spawn() {
        ClockStrategy::set(ClockStrategy::Manual);
        Instant::inc(Duration::from_nanos(1));
        thread::spawn(move || {
            assert_eq!(Instant::get(), Duration::from_nanos(1));
        })
        .join()
        .unwrap();
    }

    #[test]
    fn manual_clock_is_global() {
        ClockStrategy::set(ClockStrategy::Manual);
        let barrier = Arc::new(Barrier::new(2));
        let barrier2 = barrier.clone();
        let thread = thread::spawn(move || {
            barrier2.wait();
            assert_eq!(Instant::get(), Duration::from_nanos(1));
        });
        Instant::inc(Duration::from_nanos(1));
        barrier.wait();
        thread.join().unwrap();
    }
}
