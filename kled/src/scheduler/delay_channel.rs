use futures::{
    channel::mpsc,
    channel::mpsc::*,
    lock::Mutex,
    prelude::*,
    task::{Context, Poll},
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::pin::Pin;

#[chronobreak]
mod mock {
    pub use futures_timer::Delay;
    pub use std::sync::Arc;
    pub use std::thread;
    pub use std::time::*;
}
use mock::*;

pub struct Delayed<T> {
    t: T,
    direct: bool,
    start_time: Instant,
}

impl<T> Delayed<T> {
    pub fn new(t: T) -> Self {
        Self::with_start_time(t, Instant::now())
    }

    pub fn with_start_time(t: T, start_time: Instant) -> Self {
        Self {
            t,
            direct: false,
            start_time,
        }
    }

    pub fn with_delay(t: T, delay: Duration) -> Self {
        Self::with_start_time(t, Instant::now() + delay)
    }

    pub fn direct(t: T) -> Self {
        Self {
            t,
            direct: true,
            start_time: Instant::now(),
        }
    }

    pub fn take(self) -> T {
        self.t
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }
}

impl<T> Ord for Delayed<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.start_time.cmp(&other.start_time).reverse() {
            Ordering::Equal => {
                if self.direct {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }
            ord => ord,
        }
    }
}

impl<T> PartialOrd for Delayed<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start_time
            .partial_cmp(&other.start_time)
            .map(&Ordering::reverse)
            .map(|ord| match ord {
                Ordering::Equal => {
                    if self.direct {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                }
                ord => ord,
            })
    }
}

impl<T> Eq for Delayed<T> {}

impl<T> PartialEq for Delayed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time && self.direct == other.direct
    }
}

impl<T> From<T> for Delayed<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

pub struct DelaySender<T> {
    sender: UnboundedSender<Delayed<T>>,
}

impl<T> DelaySender<T> {
    pub async fn send_direct(&mut self, t: T) -> Result<(), <Self as Sink<T>>::Error> {
        self.sender.send(Delayed::direct(t)).await
    }

    pub async fn send_delayed(
        &mut self,
        delay: Duration,
        t: T,
    ) -> Result<(), <Self as Sink<T>>::Error> {
        self.sender.send(Delayed::with_delay(t, delay)).await
    }
}

impl<T> Sink<T> for DelaySender<T> {
    type Error = <UnboundedSender<Delayed<T>> as Sink<Delayed<T>>>::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::into_inner(self).sender.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, t: T) -> Result<(), Self::Error> {
        Pin::into_inner(self).sender.start_send(Delayed::from(t))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl<T> From<UnboundedSender<Delayed<T>>> for DelaySender<T> {
    fn from(sender: UnboundedSender<Delayed<T>>) -> Self {
        Self { sender }
    }
}

pub struct DelayReceiver<T> {
    data: Arc<Mutex<Data<T>>>,
}

impl<T> From<UnboundedReceiver<Delayed<T>>> for DelayReceiver<T> {
    fn from(receiver: UnboundedReceiver<Delayed<T>>) -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                receiver: Some(receiver),
                queue: BinaryHeap::default(),
                delay: None,
            })),
        }
    }
}

impl<T> DelayReceiver<T> {
    #[allow(dead_code)]
    pub async fn try_next(&self) -> Result<Option<T>, ()> {
        self.data.lock().await.try_next()
    }
}

impl<T> Stream for DelayReceiver<T>
where
    T: Unpin,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.data.lock().poll_unpin(cx) {
            Poll::Ready(mut lock) => lock.poll_next_unpin(cx),
            Poll::Pending => Poll::Pending,
        }
    }
}

struct Data<T> {
    receiver: Option<UnboundedReceiver<Delayed<T>>>,
    queue: BinaryHeap<Option<Delayed<T>>>,
    delay: Option<Delay>,
}

impl<T> Data<T> {
    pub fn try_next(&mut self) -> Result<Option<T>, ()> {
        self.drain_channel();
        self.try_pop().map_err(|_| ())
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        if self.delay.is_some() {
            match self.delay.as_mut().unwrap().poll_unpin(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(()) => self.delay = None,
            }
        } else {
            self.drain_channel();
            if let Ok(task) = self.try_pop() {
                return Poll::Ready(task);
            }
            if self.queue.is_empty() {
                if self.receiver.is_some() {
                    match self.receiver.as_mut().unwrap().poll_next_unpin(cx) {
                        Poll::Ready(task) => self.push(task),
                        Poll::Pending => return Poll::Pending,
                    }
                } else {
                    panic! {"DelayReceiver<T>::poll_next called after Poll::Ready(None) was returned"}
                }
            }
        }
        debug_assert! {!self.queue.is_empty()}
        self.drain_queue(cx)
    }

    fn drain_channel(&mut self) {
        while let Some(receiver) = &mut self.receiver {
            if let Ok(t) = receiver.try_next() {
                self.push(t)
            } else {
                break;
            }
        }
    }

    fn drain_queue(&mut self, cx: &mut Context<'_>) -> Poll<Option<T>> {
        loop {
            match self.try_pop() {
                Ok(t) => return Poll::Ready(t),
                Err(opt) => {
                    if let Some(mut delay) = opt {
                        match delay.poll_unpin(cx) {
                            Poll::Pending => {
                                self.delay = Some(delay);
                                return Poll::Pending;
                            }
                            Poll::Ready(()) => {}
                        }
                    } else {
                        return Poll::Pending;
                    }
                }
            }
        }
    }

    fn push(&mut self, t: Option<Delayed<T>>) {
        if t.is_none() {
            self.receiver = None;
        }
        self.queue.push(t);
    }

    fn try_pop(&mut self) -> Result<Option<T>, Option<Delay>> {
        if let Some(t) = self.queue.peek() {
            if let Some(t) = t {
                if t.start_time() <= Instant::now() {
                    Ok(self.queue.pop().unwrap().map(&Delayed::take))
                } else {
                    Err(Some(Delay::new(
                        t.start_time().saturating_duration_since(Instant::now()),
                    )))
                }
            } else {
                Ok(None)
            }
        } else {
            Err(None)
        }
    }
}

impl<T> Stream for Data<T>
where
    T: Unpin,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::into_inner(self).poll_next(cx)
    }
}

pub fn unbounded<T>() -> (DelaySender<T>, DelayReceiver<T>) {
    let (sender, receiver) = mpsc::unbounded();
    (sender.into(), receiver.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::sink::SinkExt;

    #[chronobreak::test(async, frozen)]
    async fn direct_task() {
        let (mut tx, mut rx) = unbounded();
        tx.send(0).await.unwrap();
        tx.send_direct(1).await.unwrap();
        assert_eq!(rx.next().await.unwrap(), 1);
    }

    #[chronobreak::test(async, frozen)]
    async fn non_delayed_task() {
        let (mut tx, mut rx) = unbounded();
        tx.send(0).await.unwrap();
        rx.next().await.unwrap();
    }

    #[chronobreak::test(async, frozen)]
    async fn try_non_delayed_task() {
        let (mut tx, rx) = unbounded();
        tx.send(0).await.unwrap();
        rx.try_next().await.unwrap();
    }

    #[chronobreak::test(async)]
    async fn delayed_task() {
        let (mut tx, mut rx) = unbounded();
        tx.send_delayed(Duration::from_nanos(50), 0).await.unwrap();
        rx.next().await.unwrap();
        assert_clock_eq!(Duration::from_nanos(50));
    }

    #[chronobreak::test(async)]
    async fn try_delayed_task() {
        let (mut tx, rx) = unbounded();
        tx.send_delayed(Duration::from_nanos(50), 0).await.unwrap();
        assert_clock_eq!(Duration::from_nanos(0));
        matches! {rx.try_next().await, Err(())};
        clock::advance(Duration::from_nanos(50));
        rx.try_next().await.unwrap();
    }

    #[chronobreak::test(async)]
    async fn add_direct_after_delayed_task() {
        let (mut tx, mut rx) = unbounded();
        tx.send_delayed(Duration::from_nanos(50), 0).await.unwrap();
        tx.send_delayed(Duration::from_nanos(0), 0).await.unwrap();
        assert_clock_eq!(Duration::from_nanos(0));
        rx.next().await.unwrap();
        matches! {rx.try_next().await, Err(())};
        clock::advance(Duration::from_nanos(50));
        rx.next().await.unwrap();
    }

    #[async_std::test]
    async fn test_fix_poll_after_none() {
        let (mut tx, mut rx) = unbounded();
        tx.send(0).await.unwrap();
        tx.send(0).await.unwrap();
        drop(tx);
        while rx.next().await.is_some() {}
    }
}
