#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod dev_env;
pub mod endpoint;
mod root;
#[cfg(feature = "hyper-server")]
mod server;

#[cfg(feature = "cli")]
pub use cli::Cli;
#[cfg(feature = "cli")]
pub use dev_env::DevEnv;
pub use endpoint::Endpoint;
#[cfg(feature = "hyper-server")]
pub use server::Server;

pub mod error;
mod props;
mod utils;

pub use props::ServerAppProps;
