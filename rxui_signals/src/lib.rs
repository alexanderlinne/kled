#![feature(fn_traits, unboxed_closures, trait_alias)]

use std::marker::PhantomData;
use std::sync::{Arc, Mutex, Weak};

pub trait SlotFn<Input, Output> = Fn(&Input) -> Output + Send;
pub trait CombinatorFn<T> = Fn(T, T) -> T + Send;

pub struct SignalBuilder<Input, Output> {
    combinator: Option<Box<dyn CombinatorFn<Option<Output>>>>,
    phantom: PhantomData<Input>,
}

impl<Input: 'static, Output: 'static> SignalBuilder<Input, Output> {
    pub fn new() -> Self {
        Self {
            combinator: None,
            phantom: PhantomData,
        }
    }

    pub fn with_combinator<F: 'static>(mut self, combinator: F) -> Self
    where
        F: CombinatorFn<Output>,
    {
        self.combinator = Some(Box::new(lift_combinator(combinator)));
        self
    }

    pub fn finalize(self) -> Signal<Input, Output> {
        Signal {
            data: Arc::new(Mutex::new(SignalData::new(self.combinator))),
        }
    }
}

pub struct Signal<Input, Output> {
    data: Arc<Mutex<SignalData<Input, Output>>>,
}

impl<Input: 'static, Output: 'static> Signal<Input, Output> {
    pub fn new() -> Self {
        let data = Arc::new(Mutex::new(SignalData::new(None)));
        Self { data }
    }

    pub fn connector(&self) -> SignalConnector<Input, Output> {
        SignalConnector::new(self.data.clone())
    }

    pub fn invoke(&self, arg: &Input) -> Option<Output> {
        let data = self.data.lock().unwrap();
        (&data.slots)
            .into_iter()
            .map(|slot| slot.call(&arg))
            .fold(None, &data.combinator)
    }

    pub fn connect(&mut self, slot: Box<dyn SlotFn<Input, Output>>) -> Connection {
        self.data.lock().unwrap().connect(slot)
    }

    pub fn slot_count(&self) -> usize {
        let data = self.data.lock().unwrap();
        (&data.slots)
            .into_iter()
            .filter(|slot| slot.connected())
            .count()
    }
}

struct SignalData<Input, Output> {
    slots: Vec<Arc<Mutex<ConnectionInfo<Input, Output>>>>,
    combinator: Box<dyn CombinatorFn<Option<Output>>>,
}

impl<Input: 'static, Output: 'static> SignalData<Input, Output> {
    fn new(combinator: Option<Box<dyn CombinatorFn<Option<Output>>>>) -> Self {
        let combinator = match combinator {
            Some(c) => c,
            None => Box::new(lift_combinator(|_, t| t)),
        };
        Self {
            slots: vec![],
            combinator,
        }
    }

    pub fn connect(&mut self, slot: Box<dyn SlotFn<Input, Output>>) -> Connection {
        let info = Arc::new(Mutex::new(ConnectionInfo::new(slot)));
        let base = info.clone() as Arc<Mutex<dyn ConnectionBase + Send>>;
        self.slots.push(info);
        Connection::new(Arc::downgrade(&base))
    }
}

fn lift_combinator<F: 'static, Output: 'static>(
    combinator: F,
) -> Box<dyn CombinatorFn<Option<Output>>>
where
    F: CombinatorFn<Output>,
{
    Box::new(
        move |rhs: Option<Output>, lhs: Option<Output>| match (rhs, lhs) {
            (Some(rhs), Some(lhs)) => Some(combinator(rhs, lhs)),
            (Some(rhs), _) => Some(rhs),
            (_, Some(lhs)) => Some(lhs),
            _ => None,
        },
    )
}

#[derive(Clone)]
pub struct Connection {
    connection_base: Weak<Mutex<dyn ConnectionBase + Send>>,
}

impl Connection {
    fn new(connection_base: Weak<Mutex<dyn ConnectionBase + Send>>) -> Self {
        Self { connection_base }
    }

    pub fn disconnect(&mut self) -> bool {
        match self.connection_base.upgrade() {
            Some(connection_base) => connection_base.lock().unwrap().disconnect(),
            None => true,
        }
    }
}

struct ConnectionInfo<Input, Output> {
    slot: Box<dyn SlotFn<Input, Output>>,
    connected: bool,
}

impl<Input, Output> ConnectionInfo<Input, Output> {
    fn new(slot: Box<dyn SlotFn<Input, Output>>) -> Self {
        Self {
            slot,
            connected: true,
        }
    }
}

impl<Input, Output> ConnectionBase for ConnectionInfo<Input, Output> {
    fn disconnect(&mut self) -> bool {
        self.connected = false;
        true
    }
}

impl<Input, Output> ConnectionInfoMutexHelper<Input, Output>
    for Mutex<ConnectionInfo<Input, Output>>
{
    fn call(&self, arg: &Input) -> Option<Output> {
        let info = self.lock().unwrap();
        if info.connected {
            Some((*info.slot)(arg))
        } else {
            None
        }
    }

    fn connected(&self) -> bool {
        self.lock().unwrap().connected
    }
}

trait ConnectionBase {
    fn disconnect(&mut self) -> bool;
}

trait ConnectionInfoMutexHelper<Input, Output> {
    fn call(&self, arg: &Input) -> Option<Output>;

    fn connected(&self) -> bool;
}

pub struct SignalConnector<Input, Output> {
    data: Arc<Mutex<SignalData<Input, Output>>>,
}

impl<Input: 'static, Output: 'static> SignalConnector<Input, Output> {
    fn new(data: Arc<Mutex<SignalData<Input, Output>>>) -> Self {
        Self { data }
    }

    pub fn connect(&mut self, slot: Box<dyn SlotFn<Input, Output>>) -> Connection {
        self.data.lock().unwrap().connect(slot)
    }
}

pub struct ScopedConnection {
    connection: Connection,
}

impl ScopedConnection {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }
}

impl Drop for ScopedConnection {
    fn drop(&mut self) {
        self.connection.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use crate::{ScopedConnection, Signal, SignalBuilder};
    use std::thread;

    #[test]
    fn basic_connections() {
        let mut signal = Signal::<i32, i32>::new();
        assert_eq!(signal.invoke(&1), None);
        signal.connect(Box::new(|v| v + 1));
        assert_eq!(signal.invoke(&1), Some(2));
        let mut connection = signal.connect(Box::new(|v| v + 2));
        assert_eq!(signal.invoke(&1), Some(3));
        connection.disconnect();
        assert_eq!(signal.invoke(&1), Some(2));
    }

    #[test]
    fn slot_count() {
        let mut signal = Signal::<i32, i32>::new();
        assert_eq!(signal.slot_count(), 0);
        signal.connect(Box::new(|v| v + 1));
        assert_eq!(signal.slot_count(), 1);
        let mut connection = signal.connect(Box::new(|v| v + 2));
        assert_eq!(signal.slot_count(), 2);
        connection.disconnect();
        assert_eq!(signal.slot_count(), 1);
    }

    #[test]
    fn connection_and_signal_are_send() {
        let mut signal = Signal::<i32, i32>::new();
        let connection = signal.connect(Box::new(|v| v + 1));
        let _thread = thread::spawn(move || {
            let _conn = connection;
        });
        let _thread = thread::spawn(move || {
            let _sig = signal;
        });
    }

    #[test]
    fn combinator() {
        let mut signal = SignalBuilder::<i32, i32>::new()
            .with_combinator(|a, b| a + b)
            .finalize();
        signal.connect(Box::new(|v| *v));
        signal.connect(Box::new(|v| *v));
        assert_eq!(signal.invoke(&1), Some(2));
    }

    #[test]
    fn scoped_connection() {
        let mut signal = Signal::<i32, i32>::new();
        assert_eq!(signal.invoke(&1), None);
        let connection = signal.connect(Box::new(|v| *v));
        assert_eq!(signal.invoke(&1), Some(1));
        {
            let _scoped = ScopedConnection::new(connection);
            assert_eq!(signal.invoke(&1), Some(1));
        }
        assert_eq!(signal.invoke(&1), None);
    }

    #[test]
    fn signal_connector() {
        let mut signal = Signal::<i32, i32>::new();
        assert_eq!(signal.invoke(&1), None);
        signal.connect(Box::new(|v| v + 1));
        assert_eq!(signal.invoke(&1), Some(2));
        let mut connector = signal.connector();
        let mut connection = connector.connect(Box::new(|v| v + 2));
        assert_eq!(signal.invoke(&1), Some(3));
        connection.disconnect();
        assert_eq!(signal.invoke(&1), Some(2));
    }
}
