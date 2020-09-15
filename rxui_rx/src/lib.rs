//! [![github]](https://github.com/alexanderlinne/rxui)&ensp;
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
#![deny(intra_doc_link_resolution_failure)]

#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate rxui_rx_derive;
extern crate threadpool;

pub mod cancellable;
pub mod core;
pub mod emitter;
pub mod flow;
pub mod marker;
pub mod observable;
pub mod observer;
#[doc(hidden)]
pub mod operators;
pub mod scheduler;
pub mod subject;
pub mod subscription;
pub mod util;

#[doc(hidden)]
pub mod prelude;
