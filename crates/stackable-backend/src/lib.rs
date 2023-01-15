//! Stackable backend server.

#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

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
