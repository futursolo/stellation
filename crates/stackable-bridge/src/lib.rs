#[cfg(feature = "resolvable")]
mod bridge_local;
mod bridge_remote;
pub mod error;
pub mod hooks;
#[cfg(feature = "resolvable")]
pub mod resolvers;
pub mod state;
pub mod types;

#[cfg(feature = "resolvable")]
pub use bridge_local::LocalBridge;
#[cfg(feature = "resolvable")]
pub use bridge_local::LocalBridge as Bridge;
pub use bridge_remote::RemoteBridge;
#[cfg(not(feature = "resolvable"))]
pub use bridge_remote::RemoteBridge as Bridge;
