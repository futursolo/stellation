use std::marker::PhantomData;

use async_trait::async_trait;

use super::Link;
use crate::routines::{BridgedMutation, BridgedQuery, MutationResult, QueryResult};
use crate::BridgeResult;

/// A Link that does nothing.
///
/// This is used as a type parameter for types that may or may not have a link.
#[derive(Debug, Clone)]
pub struct PhantomLink {
    _marker: PhantomData<()>,
}

impl PartialEq for PhantomLink {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[async_trait(?Send)]
impl Link for PhantomLink {
    async fn resolve_encoded(&self, _input_buf: &[u8]) -> BridgeResult<Vec<u8>> {
        unimplemented!()
    }

    async fn resolve_query<T>(&self, _input: &T::Input) -> QueryResult<T>
    where
        T: 'static + BridgedQuery,
    {
        unimplemented!()
    }

    async fn resolve_mutation<T>(&self, _input: &T::Input) -> MutationResult<T>
    where
        T: 'static + BridgedMutation,
    {
        unimplemented!()
    }
}
