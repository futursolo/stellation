use std::any::TypeId;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("failed to communicate with server")]
    Network(#[from] gloo_net::Error),
    #[error("failed to encode / decode content")]
    Encoding(#[from] bincode::Error),
    #[error("failed to find type with index: {}", .0)]
    InvalidIndex(usize),
    #[error("failed to find type: {:?}", .0)]
    InvalidType(TypeId),
}
pub type BridgeResult<T> = Result<T, BridgeError>;
