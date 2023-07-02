//! Registries for Routines and their Resolvers.

use serde::{Deserialize, Serialize};

mod routine;
pub use routine::*;

mod resolver;
pub use resolver::*;

#[derive(Debug, Serialize, Deserialize)]
struct Incoming<'a> {
    query_index: usize,
    input: &'a [u8],
}
