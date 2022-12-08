use serde::{Deserialize, Serialize};
use stackable_bridge::types::{BridgedQuery, Never};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerTimeQuery {
    pub value: OffsetDateTime,
}

impl BridgedQuery for ServerTimeQuery {
    type Error = Never;
    type Input = ();
}
