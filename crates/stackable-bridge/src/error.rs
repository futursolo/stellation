use thiserror::Error;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("failed to communicate with server")]
    Network(#[from] gloo_net::Error),
    #[error("failed to encode / decode content")]
    Encoding(#[from] bincode::Error),
}
