//! Stellation's wrap support.

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
mod filters;
mod frontend;
mod html;
mod request;
mod utils;

pub use endpoint::WarpEndpoint;
pub use frontend::Frontend;
use once_cell::sync::Lazy;
pub use request::WarpRequest;

// A server id that is different every time it starts.
static SERVER_ID: Lazy<String> = Lazy::new(crate::utils::random_str);
