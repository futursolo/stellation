#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "cli")]
mod cli;
mod endpoint;
mod props;
mod root;
#[cfg(feature = "hyper-server")]
mod server;
pub mod trace;

#[cfg(feature = "cli")]
pub use cli::Cli;
pub use endpoint::Endpoint;
pub use props::ServerAppProps;
#[cfg(feature = "hyper-server")]
pub use server::Server;

pub mod error;
pub mod utils;
