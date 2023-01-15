use async_trait::async_trait;
use stackable_bridge::resolvers::{MutationResolver, QueryResolver};
use stackable_bridge::types::{MutationResult, QueryResult};
use stackable_bridge::BridgeMetadata;
use time::OffsetDateTime;

use crate::types::*;

#[async_trait(?Send)]
impl QueryResolver for ServerTimeQuery {
    type Context = ();

    async fn resolve(_metadata: &BridgeMetadata<()>, _input: &Self::Input) -> QueryResult<Self> {
        Ok(Self {
            value: OffsetDateTime::now_utc(),
        }
        .into())
    }
}

#[async_trait(?Send)]
impl MutationResolver for GreetingMutation {
    type Context = ();

    async fn resolve(_metadata: &BridgeMetadata<()>, name: &Self::Input) -> MutationResult<Self> {
        Ok(Self {
            message: format!("Hello, {name}!"),
        }
        .into())
    }
}
