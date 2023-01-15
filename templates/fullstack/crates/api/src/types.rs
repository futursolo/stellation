use serde::{Deserialize, Serialize};
use stackable_bridge::types::{BridgedMutation, BridgedQuery};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerTimeQuery {
    pub value: OffsetDateTime,
}

#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Error {
    #[error("failed to communicate with server.")]
    Network,
}

impl BridgedQuery for ServerTimeQuery {
    type Error = Error;
    type Input = ();

    fn into_query_error(_e: stackable_bridge::BridgeError) -> Self::Error {
        Error::Network
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GreetingMutation {
    pub message: String,
}

impl BridgedMutation for GreetingMutation {
    type Error = Error;
    type Input = String;

    fn into_mutation_error(_e: stackable_bridge::BridgeError) -> Self::Error {
        Error::Network
    }
}
