use async_trait::async_trait;

use crate::types::{Mutation, MutationResult, Query, QueryResult};

#[async_trait]
pub trait QueryResolver: Query {
    async fn resolve(input: &Self::Input) -> QueryResult<Self>;
}

#[async_trait]
pub trait MutationResolver: Mutation {
    async fn resolve(input: &Self::Input) -> MutationResult<Self>;
}
