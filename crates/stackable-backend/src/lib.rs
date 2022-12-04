#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "cli")]
mod cli;
mod endpoint;
#[cfg(feature = "hyper-server")]
mod server;

#[cfg(feature = "cli")]
mod dev_env;

#[cfg(feature = "cli")]
pub use cli::Cli;
pub use endpoint::Endpoint;
#[cfg(feature = "hyper-server")]
pub use server::Server;
