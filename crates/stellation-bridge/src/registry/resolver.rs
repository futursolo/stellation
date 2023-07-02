use std::fmt;
use std::sync::Arc;

use futures::future::{self, LocalBoxFuture};
use futures::{FutureExt, TryFutureExt};

use super::Incoming;
use crate::resolvers::{MutationResolver, QueryResolver};
use crate::{BridgeError, BridgeMetadata, BridgeResult};

pub(super) type Resolver<CTX> = Arc<
    dyn Send
        + Sync
        + Fn(&BridgeMetadata<CTX>, &[u8]) -> LocalBoxFuture<'static, BridgeResult<Vec<u8>>>,
>;

pub(super) type Resolvers<CTX> = Vec<Resolver<CTX>>;

/// The Registry Builder for Resolver Registry
pub struct ResolverRegistryBuilder<CTX = ()> {
    resolvers: Resolvers<CTX>,
}

impl fmt::Debug for ResolverRegistryBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolverRegistryBuilder")
            .finish_non_exhaustive()
    }
}

impl<CTX> Default for ResolverRegistryBuilder<CTX> {
    fn default() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }
}

impl<CTX> ResolverRegistryBuilder<CTX>
where
    CTX: 'static,
{
    /// Creates a registry builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a registry.
    pub fn build(self) -> ResolverRegistry<CTX> {
        ResolverRegistry {
            inner: Arc::new(self),
        }
    }

    /// Adds a Query Resolver
    pub fn add_query<T>(mut self) -> Self
    where
        T: 'static + QueryResolver<Context = CTX>,
    {
        let resolver = Arc::new(|metadata: &BridgeMetadata<CTX>, input: &[u8]| {
            let metadata = metadata.duplicate();
            let input = match bincode::deserialize::<T::Input>(input)
                .map_err(BridgeError::Encoding)
                .map_err(future::err)
                .map_err(|e| e.boxed_local())
            {
                Ok(m) => m,
                Err(e) => return e,
            };
            async move { T::resolve(&metadata, &input).await }
                .map(|m| bincode::serialize(&m.as_deref()))
                .map_err(BridgeError::Encoding)
                .boxed_local()
        });

        self.resolvers.push(resolver);
        self
    }

    /// Adds a Mutation Resolver
    pub fn add_mutation<T>(mut self) -> Self
    where
        T: 'static + MutationResolver<Context = CTX>,
    {
        let resolver = Arc::new(|metadata: &BridgeMetadata<CTX>, input: &[u8]| {
            let metadata = metadata.duplicate();
            let input = match bincode::deserialize::<T::Input>(input)
                .map_err(BridgeError::Encoding)
                .map_err(future::err)
                .map_err(|e| e.boxed_local())
            {
                Ok(m) => m,
                Err(e) => return e,
            };
            async move { T::resolve(&metadata, &input).await }
                .map(|m| bincode::serialize(&m.as_deref()))
                .map_err(BridgeError::Encoding)
                .boxed_local()
        });

        self.resolvers.push(resolver);
        self
    }
}

/// The Registry that holds available query and mutation resolvers.
pub struct ResolverRegistry<CTX> {
    inner: Arc<ResolverRegistryBuilder<CTX>>,
}

impl<CTX> fmt::Debug for ResolverRegistry<CTX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolverRegistry").finish_non_exhaustive()
    }
}

impl<CTX> Clone for ResolverRegistry<CTX> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<CTX> ResolverRegistry<CTX> {
    /// Creates a Builder for remote registry.
    pub fn builder() -> ResolverRegistryBuilder {
        ResolverRegistryBuilder::new()
    }

    /// Resolves an encoded request.
    pub async fn resolve_encoded(
        &self,
        metadata: &BridgeMetadata<CTX>,
        incoming: &[u8],
    ) -> BridgeResult<Vec<u8>> {
        let incoming: Incoming<'_> = bincode::deserialize(incoming)?;

        let resolver = self
            .inner
            .resolvers
            .get(incoming.query_index)
            .ok_or(BridgeError::InvalidIndex(incoming.query_index))?;

        resolver(metadata, incoming.input).await
    }
}
