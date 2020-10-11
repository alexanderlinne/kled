use crate::core;

pub trait Subject<Cancellable, CancellableOut, Item, Error>:
    core::Observable<CancellableOut, Item, Error> + core::Observer<Cancellable, Item, Error>
{
}
