use serde::{Deserialize, Serialize};
use stackable_bridge::types::{Never, Query};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerTimeQuery {
    pub value: OffsetDateTime,
}

impl Query for ServerTimeQuery {
    type Error = Never;
    type Input = ();
}
