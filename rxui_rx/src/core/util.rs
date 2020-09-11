use crate::core;

pub trait Sealed {}

impl<'o, Observable> Sealed for Observable where Observable: core::LocalObservable<'o> + 'o {}

impl<Observable> Sealed for core::Shared<Observable> where
    Observable: core::SharedObservable + Send + 'static
{
}

pub trait Inconstructible: Sealed {}

#[derive(Copy, Clone)]
pub enum Infallible {}

impl Sealed for Infallible {}

impl Inconstructible for Infallible {}
