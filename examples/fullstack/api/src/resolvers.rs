use async_trait::async_trait;
use stackable_bridge::resolvers::QueryResolver;
use stackable_bridge::types::QueryResult;
use time::OffsetDateTime;

use crate::types::*;

#[async_trait]
impl QueryResolver for ServerTimeQuery {
    async fn resolve(_input: &Self::Input) -> QueryResult<Self> {
        Ok(Self {
            value: OffsetDateTime::now_utc(),
        }
        .into())
    }
}
