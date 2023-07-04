//! Bridge between the frontend and backend.
//!
//! This module is a RPC implementation that facilitates communications between frontend and
//! backend.
//!
//! It supports the following routines:
//!
//! - [Query](types::BridgedQuery)
//! - [Mutation](types::BridgedMutation)
//!
//! Bridge has 2 connection methods `local` and `remote`. When a `LocalLink` is used, routines will
//! be connected with the local method and can process requests with resolvers. This can be used for
//! server-side rendering and processing requests from a bridge connected with the remote method. If
//! the `FetchLink` is used, it will send the request to the bridge endpoint which will
//! process the routine at the server-side. This is usually used for client-side rendering.
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
pub mod routines;
pub mod state;

pub use bridge::Bridge;
pub use error::{BridgeError, BridgeResult};
