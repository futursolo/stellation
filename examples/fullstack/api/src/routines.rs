use serde::{Deserialize, Serialize};
use stellation_bridge::links::FetchLink;
use stellation_bridge::registry::RoutineRegistry;
use stellation_bridge::routines::{BridgedMutation, BridgedQuery};
use stellation_bridge::Bridge as Bridge_;
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

    fn into_query_error(_e: stellation_bridge::BridgeError) -> Self::Error {
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

    fn into_mutation_error(_e: stellation_bridge::BridgeError) -> Self::Error {
        Error::Network
    }
}
pub fn create_routine_registry() -> RoutineRegistry {
    RoutineRegistry::builder()
        .add_query::<ServerTimeQuery>()
        .add_mutation::<GreetingMutation>()
        .build()
}

pub type Link = FetchLink;
pub type Bridge = Bridge_<Link>;

pub fn create_frontend_bridge() -> Bridge {
    Bridge::new(
        FetchLink::builder()
            .routines(create_routine_registry())
            .build(),
    )
}
