//! Stellation's tower support.

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
pub use endpoint::TowerEndpoint;
/// A stellation request with information extracted with tower services.
///
/// Currently, this is a type alias to [`WarpRequest`](stellation_backend_warp::WarpRequest).
pub type TowerRequest<CTX> = stellation_backend_warp::WarpRequest<CTX>;
#[doc(inline)]
pub use stellation_backend_warp::Frontend;

mod server;
pub use server::Server;
