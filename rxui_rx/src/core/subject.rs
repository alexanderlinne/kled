use crate::core;

pub trait LocalSubject<'o, ObservationIn, Item, Error>:
    core::LocalObservable<'o> + core::Observer<ObservationIn, Item, Error>
{
}

pub trait SharedSubject<ObservationIn, Item, Error>:
    core::SharedObservable + core::Observer<ObservationIn, Item, Error>
{
}
