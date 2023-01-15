//! Stackable Backend
//!
//! This crate contains the backend server and utilities for backend.

#![deny(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(unsafe_code)]
#![deny(non_snake_case)]
#![deny(clippy::cognitive_complexity)]
#![deny(missing_docs)]
#![cfg_attr(documenting, feature(doc_cfg))]
#![cfg_attr(documenting, feature(doc_auto_cfg))]
#![cfg_attr(any(releasing, not(debug_assertions)), deny(dead_code, unused_imports))]

mod endpoint;
mod error;
mod props;
mod root;
pub mod trace;
pub mod utils;
pub use endpoint::Endpoint;
pub use error::{ServerAppError, ServerAppResult};
pub use props::ServerAppProps;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
pub use cli::Cli;

#[cfg(feature = "warp-filter")]
mod frontend;
#[cfg(feature = "warp-filter")]
pub use frontend::Frontend;

#[cfg(feature = "hyper-server")]
mod server;
#[cfg(feature = "hyper-server")]
pub use server::Server;
