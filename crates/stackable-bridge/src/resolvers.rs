use async_trait::async_trait;

use crate::types::{BridgedMutation, BridgedQuery, MutationResult, QueryResult};

#[async_trait(?Send)]
pub trait QueryResolver: BridgedQuery {
    async fn resolve(input: &Self::Input) -> QueryResult<Self>;
}

#[async_trait(?Send)]
pub trait MutationResolver: BridgedMutation {
    async fn resolve(input: &Self::Input) -> MutationResult<Self>;
}
