//! Bridge between the frontend and backend.
//!
//! This module is a RPC implementation that facilitates communications between frontend and
//! backend.
//!
//! It supports the following types:
//!
//! - [Query](types::BridgedQuery)
//! - [Mutation](types::BridgedMutation)
//!
//! Bridge has 2 connection methods `local` and `remote`. When the `resolvable` feature is
//! enabled, bridges will be conntected with the local method and can process requests with
//! resolvers. This can be used for server-side rendering and processing requests from a bridge
//! connected with the remote method. If the `resolvable` feature is disabled, it will send the
//! request to the bridge endpoint which will process the bridge at the server-side. This is usually
//! used for client-side rendering.
//!
//! When the `resolvable` feature is enabled, it will require query and mutations to implement their
//! resolver type which is responsible for resolving the request.
//!
//! You can check out the [example](https://github.com/futursolo/stellation/blob/main/examples/fullstack/api/src/lib.rs) for how to implement resolvers.

#![deny(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(unsafe_code)]
#![deny(non_snake_case)]
#![deny(clippy::cognitive_complexity)]
#![deny(missing_docs)]
#![cfg_attr(documenting, feature(doc_cfg))]
#![cfg_attr(documenting, feature(doc_auto_cfg))]
#![cfg_attr(any(releasing, not(debug_assertions)), deny(dead_code, unused_imports))]

mod bridge;
mod error;
pub mod hooks;
pub mod links;
pub mod registry;
pub mod resolvers;
pub mod state;
pub mod types;

pub use bridge::{Bridge, BridgeBuilder, BridgeMetadata, ConnectedBridge};
pub use error::{BridgeError, BridgeResult};
