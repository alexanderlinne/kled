use crate::core;

pub trait Subject<Cancellable, CancellableOut, Item, Error>:
    core::Observable<CancellableOut, Item, Error> + core::Observer<Cancellable, Item, Error>
where
    CancellableOut: core::Cancellable + Send + Sync + 'static,
    Item: Send + 'static,
    Error: Send + 'static,
{
}
