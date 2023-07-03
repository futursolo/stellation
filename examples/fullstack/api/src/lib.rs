#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "resolvable")]
mod resolvers;
mod types;

#[cfg(feature = "resolvable")]
pub use resolvers::*;
#[cfg(not(feature = "resolvable"))]
pub use types::*;
