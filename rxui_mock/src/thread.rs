use crate::sync::{Arc, Mutex};
use crate::time::*;
use std::thread;

pub use std::thread::{panicking, Builder};

pub fn sleep(dur: Duration) {
    match ClockStrategy::current() {
        ClockStrategy::Sys => thread::sleep(dur),
        ClockStrategy::Manual => {}
        ClockStrategy::AutoInc => Instant::inc(dur),
    }
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let clock_mode = ClockStrategy::current_raw();
    let manual_nanos = MANUAL_NANOS.with(|v| v.borrow().clone());
    let auto_inc = AUTO_INC_NANOS.with(|v| *v.borrow());
    let join_cell = Arc::new(Mutex::new(None));
    let join_cell_weak = Arc::downgrade(&join_cell);
    let handle = thread::spawn(move || {
        if let Some(clock_mode) = clock_mode {
            ClockStrategy::set(clock_mode);
            MANUAL_NANOS.with(|v| *v.borrow_mut() = manual_nanos);
            AUTO_INC_NANOS.with(|v| *v.borrow_mut() = auto_inc);
        }
        let result = f();
        if let Some(cell) = join_cell_weak.upgrade() {
            *cell.lock() = Some(AUTO_INC_NANOS.with(|v| *v.borrow()));
        }
        result
    });
    JoinHandle(join_cell, handle)
}

pub struct JoinHandle<T>(Arc<Mutex<Option<Duration>>>, thread::JoinHandle<T>);

impl<T> JoinHandle<T> {
    pub fn thread(&self) -> &thread::Thread {
        self.1.thread()
    }

    pub fn join(self) -> thread::Result<T> {
        let result = self.1.join();
        if let Some(auto_inc) = *self.0.lock() {
            AUTO_INC_NANOS.with(|v| *v.borrow_mut() = auto_inc);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::thread;
    use crate::time::*;

    #[test]
    fn auto_inc_transfers_on_thread_spawn() {
        ClockStrategy::set(ClockStrategy::AutoInc);
        Instant::inc(Duration::from_nanos(1));
        thread::spawn(move || {
            assert_eq!(Instant::get(), Duration::from_nanos(1));
        })
        .join()
        .unwrap();
    }

    #[test]
    fn auto_inc_thread_sleep() {
        ClockStrategy::set(ClockStrategy::AutoInc);
        thread::sleep(Duration::from_nanos(1));
        assert_eq!(Instant::get(), Duration::from_nanos(1));
    }

    #[test]
    fn auto_inc_is_not_global() {
        ClockStrategy::set(ClockStrategy::AutoInc);
        // Don't use mock thread::spawn here!
        std::thread::spawn(move || {
            thread::sleep(Duration::from_nanos(1));
        })
        .join()
        .unwrap();
        assert_eq!(Instant::get(), Duration::from_nanos(0));
    }

    #[test]
    fn auto_inc_thread_join_sync() {
        ClockStrategy::set(ClockStrategy::AutoInc);
        thread::spawn(move || {
            thread::sleep(Duration::from_nanos(1));
        })
        .join()
        .unwrap();
        assert_eq!(Instant::get(), Duration::from_nanos(1));
    }
}
