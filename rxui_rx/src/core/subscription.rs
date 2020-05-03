pub trait Subscription {
    fn cancel(self);

    fn request(&self, count: usize);
}
