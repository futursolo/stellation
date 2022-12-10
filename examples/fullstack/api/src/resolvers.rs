use async_trait::async_trait;
use stackable_bridge::resolvers::{MutationResolver, QueryResolver};
use stackable_bridge::types::{MutationResult, QueryResult};
use time::OffsetDateTime;

use crate::types::*;

#[async_trait(?Send)]
impl QueryResolver for ServerTimeQuery {
    async fn resolve(_input: &Self::Input) -> QueryResult<Self> {
        Ok(Self {
            value: OffsetDateTime::now_utc(),
        }
        .into())
    }
}

#[async_trait(?Send)]
impl MutationResolver for GreetingMutation {
    async fn resolve(name: &Self::Input) -> MutationResult<Self> {
        Ok(Self {
            message: format!("Hello, {name}!"),
        }
        .into())
    }
}
