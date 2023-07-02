//! The links used to resolve routines.
//!
//! For server-sided links, a new link should be created for each connection.

use async_trait::async_trait;

use crate::types::{BridgedMutation, BridgedQuery, MutationResult, QueryResult};
mod fetch_link;
mod local_link;

pub use fetch_link::FetchLink;
pub use local_link::LocalLink;

/// Common methods across all links.
#[async_trait(?Send)]
pub trait Link {
    /// Resolves a Query.
    async fn resolve_query<T>(&self, input: &T::Input) -> QueryResult<T>
    where
        T: 'static + BridgedQuery;

    /// Resolves a Mutation.
    async fn resolve_mutation<T>(&self, input: &T::Input) -> MutationResult<T>
    where
        T: 'static + BridgedMutation;
}
