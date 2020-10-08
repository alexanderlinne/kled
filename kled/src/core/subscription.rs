pub trait Subscription {
    fn cancel(&self);

    fn is_cancelled(&self) -> bool;

    fn request(&self, count: usize);
}

pub trait SubscriptionProvider {
    type Subscription: Subscription;

    fn subscription(&self) -> Self::Subscription;
}

impl<'o> Subscription for Box<dyn Subscription + 'o> {
    fn cancel(&self) {
        (&**self).cancel()
    }

    fn is_cancelled(&self) -> bool {
        (&**self).is_cancelled()
    }

    fn request(&self, count: usize) {
        (&**self).request(count)
    }
}

impl Subscription for Box<dyn Subscription + Send + 'static> {
    fn cancel(&self) {
        (&**self).cancel()
    }

    fn is_cancelled(&self) -> bool {
        (&**self).is_cancelled()
    }

    fn request(&self, count: usize) {
        (&**self).request(count)
    }
}
