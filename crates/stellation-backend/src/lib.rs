//! Stellation Backend
//!
//! This crate contains the server renderer and tools used for server-side rendering.

#![deny(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(unsafe_code)]
#![deny(non_snake_case)]
#![deny(clippy::cognitive_complexity)]
#![deny(missing_docs)]
#![cfg_attr(documenting, feature(doc_cfg))]
#![cfg_attr(documenting, feature(doc_auto_cfg))]
#![cfg_attr(any(releasing, not(debug_assertions)), deny(dead_code, unused_imports))]

mod error;
mod props;
mod root;
pub mod utils;
pub use error::{ServerAppError, ServerAppResult};
pub use props::ServerAppProps;
mod request;
pub use request::{RenderRequest, Request};
mod renderer;
pub use renderer::ServerRenderer;
mod html;
