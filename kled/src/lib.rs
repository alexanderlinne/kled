//! [![github]](https://github.com/alexanderlinne/kled)&ensp;
//!
//! [github]: https://img.shields.io/github/workflow/status/alexanderlinne/kled/CI?style=for-the-badge&logo=github
//!
#![deny(broken_intra_doc_links)]

#[macro_use]
extern crate blanket;
#[macro_use]
extern crate chronobreak;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate kled_derive;

pub mod cancellable;
pub mod core;
pub mod emitter;
pub mod flow;
pub mod marker;
pub mod observable;
pub mod observer;
pub mod scheduler;
pub mod subject;
pub mod subscriber;
pub mod subscription;
pub mod util;

#[doc(hidden)]
pub mod prelude;
