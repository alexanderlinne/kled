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
pub mod util;

pub mod cancellable;
pub mod core;
pub mod flow;
pub mod marker;
pub mod observable;
pub mod observer;
pub mod scheduler;
pub mod subject;
pub mod subscriber;
pub mod subscription;

#[doc(hidden)]
pub mod prelude;
