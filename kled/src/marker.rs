pub trait Observable<Cancellable, Item, Error> {}

impl<T, Cancellable, Item, Error> Observable<Cancellable, Item, Error> for T {}
