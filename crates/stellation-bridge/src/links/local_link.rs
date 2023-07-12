use std::cell::Cell;
use std::sync::Arc;

use async_trait::async_trait;
use futures::{future, FutureExt, TryFutureExt};
use typed_builder::TypedBuilder;

use super::Link;
use crate::registry::{ResolverRegistry, RoutineRegistry};
use crate::routines::{BridgedMutation, BridgedQuery, MutationResult, QueryResult};
use crate::BridgeResult;

/// A Link that resolves routine with local resolvers.
///
/// This is usually used to implement SSR or a backend server.
#[derive(TypedBuilder, Debug)]
pub struct LocalLink<CTX = ()> {
    /// The routine registry for all registered routines.
    routines: RoutineRegistry,
    /// The routine registry for all registered resolvers.
    resolvers: ResolverRegistry<CTX>,

    /// The bridge context
    #[builder(setter(into))]
    context: Arc<CTX>,

    /// The link equity tracker.
    #[builder(setter(skip), default_code = r#"LocalLink::<()>::next_id()"#)]
    id: usize,
}

impl<CTX> PartialEq for LocalLink<CTX> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<CTX> Clone for LocalLink<CTX> {
    fn clone(&self) -> Self {
        Self {
            routines: self.routines.clone(),
            resolvers: self.resolvers.clone(),
            context: self.context.clone(),
            id: self.id,
        }
    }
}

impl<CTX> LocalLink<CTX> {
    fn next_id() -> usize {
        thread_local! {
            static ID: Cell<usize> = Cell::new(0);
        }

        ID.with(|m| {
            m.set(m.get() + 1);

            m.get()
        })
    }
}

#[async_trait(?Send)]
impl<CTX> Link for LocalLink<CTX> {
    async fn resolve_encoded(&self, input_buf: &[u8]) -> BridgeResult<Vec<u8>> {
        self.resolvers
            .resolve_encoded(&self.context, input_buf)
            .await
    }

    async fn resolve_query<T>(&self, input: &T::Input) -> QueryResult<T>
    where
        T: 'static + BridgedQuery,
    {
        future::ready(input)
            .map(|m| self.routines.encode_query_input::<T>(m))
            .and_then(|m| async move { self.resolvers.resolve_encoded(&self.context, &m).await })
            .map_err(T::into_query_error)
            .and_then(|m| async move { self.routines.decode_query_output::<T>(&m) })
            .await
    }

    async fn resolve_mutation<T>(&self, input: &T::Input) -> MutationResult<T>
    where
        T: 'static + BridgedMutation,
    {
        future::ready(input)
            .map(|m| self.routines.encode_mutation_input::<T>(m))
            .and_then(|m| async move { self.resolvers.resolve_encoded(&self.context, &m).await })
            .map_err(T::into_mutation_error)
            .and_then(|m| async move { self.routines.decode_mutation_output::<T>(&m) })
            .await
    }
}
