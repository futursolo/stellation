use async_trait::async_trait;

use crate::types::{BridgedMutation, BridgedQuery, MutationResult, QueryResult};
use crate::BridgeMetadata;

#[async_trait(?Send)]
pub trait QueryResolver: BridgedQuery {
    type Context: 'static;

    async fn resolve(
        meta: &BridgeMetadata<Self::Context>,
        input: &Self::Input,
    ) -> QueryResult<Self>;
}

#[async_trait(?Send)]
pub trait MutationResolver: BridgedMutation {
    type Context: 'static;

    async fn resolve(
        meta: &BridgeMetadata<Self::Context>,
        input: &Self::Input,
    ) -> MutationResult<Self>;
}
