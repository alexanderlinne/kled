pub mod core;
pub mod observable;
pub mod observer;
pub mod subscriber;

#[cfg(test)]
mod test {
    use crate::core::*;
    use crate::observable;
    use crate::observer;
    use std::thread;

    #[test]
    fn test() {
        let observable = observable::create(|mut subscriber| {
            if !subscriber.is_unsubscribed() {
                subscriber.on_next(1);
            }
        });
        observable.subscribe(observer::from_next_fn(|v| {
            println!("{}", v);
        }));
    }

    #[test]
    fn test_send() {
        let observable = observable::create(|mut subscriber| {
            if !subscriber.is_unsubscribed() {
                subscriber.on_next(1);
            }
        });
        thread::spawn(move || observable.subscribe(observer::from_next_fn(|v| {
            println!("{}", v);
        })));
    }
}
