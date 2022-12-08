use async_trait::async_trait;

use crate::error::BridgeError;
use crate::types::{BridgedMutation, BridgedQuery, MutationResult, QueryResult};

#[cold]
fn panic_network_error(e: BridgeError) -> ! {
    panic!("failed to communicate with server: {:?}", e);
}

#[async_trait(?Send)]
pub trait QueryResolver: BridgedQuery {
    async fn resolve(input: &Self::Input) -> QueryResult<Self>;

    #[cold]
    fn into_query_error(e: BridgeError) -> Self::Error {
        panic_network_error(e);
    }
}

#[async_trait(?Send)]
pub trait MutationResolver: BridgedMutation {
    async fn resolve(input: &Self::Input) -> MutationResult<Self>;

    #[cold]
    fn into_mutation_error(e: BridgeError) -> Self::Error {
        panic_network_error(e);
    }
}
