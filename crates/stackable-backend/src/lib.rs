#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

mod endpoint;
#[cfg(feature = "hyper-server")]
mod server;

pub use endpoint::Endpoint;
#[cfg(feature = "hyper-server")]
pub use server::Server;
