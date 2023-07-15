//! The stylist integration for stellation.
//!
//! You can check out this [example](https://github.com/futursolo/stellation/tree/main/examples/fullstack) for how to use this crate.

#![deny(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(unsafe_code)]
#![deny(non_snake_case)]
#![deny(clippy::cognitive_complexity)]
#![deny(missing_docs)]
#![cfg_attr(documenting, feature(doc_cfg))]
#![cfg_attr(documenting, feature(doc_auto_cfg))]
#![cfg_attr(any(releasing, not(debug_assertions)), deny(dead_code, unused_imports))]

#[cfg(feature = "backend")]
mod backend;
#[cfg(feature = "frontend")]
mod frontend;

#[cfg(feature = "backend")]
pub use backend::BackendManagerProvider;
#[cfg(feature = "frontend")]
pub use frontend::FrontendManagerProvider;
