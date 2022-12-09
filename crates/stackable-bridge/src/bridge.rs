use std::fmt;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::error::{BridgeError, BridgeResult};
use crate::types::{MutationResult, QueryResult};

#[cfg(feature = "resolvable")]
mod feat_resolvable {
    use std::sync::Arc;

    use futures::future::LocalBoxFuture;
    use futures::FutureExt;

    use super::*;
    pub(super) use crate::resolvers::{
        MutationResolver as BridgedMutation, QueryResolver as BridgedQuery,
    };

    pub(super) type Resolvers =
        Vec<Arc<dyn Send + Sync + Fn(&[u8]) -> LocalBoxFuture<'static, BridgeResult<Vec<u8>>>>>;

    #[derive(Default)]
    pub struct BridgeBuilder {
        resolvers: Resolvers,
    }

    impl Bridge {
        pub async fn resolve_query<T>(&self, input: &T::Input) -> QueryResult<T>
        where
            T: 'static + BridgedQuery,
        {
            T::resolve(input).await
        }

        pub async fn resolve_mutation<T>(&self, input: &T::Input) -> MutationResult<T>
        where
            T: 'static + BridgedMutation,
        {
            T::resolve(input).await
        }

        pub async fn resolve_encoded(&self, idx: usize, input: &[u8]) -> BridgeResult<Vec<u8>> {
            let resolver = self
                .inner
                .resolvers
                .get(idx)
                .ok_or(BridgeError::InvalidIndex(idx))?;

            resolver(input).await
        }
    }

    impl BridgeBuilder {
        pub fn add_query<T>(mut self) -> Self
        where
            T: 'static + BridgedQuery,
        {
            let resolver = Arc::new(|input: &[u8]| {
                let input = match bincode::deserialize::<T::Input>(input) {
                    Ok(m) => m,
                    Err(e) => return async move { Err(BridgeError::Encoding(e)) }.boxed_local(),
                };

                async move {
                    let result = T::resolve(&input).await;
                    bincode::serialize(&result.as_deref()).map_err(BridgeError::Encoding)
                }
                .boxed_local()
            });

            self.resolvers.push(resolver);
            self
        }

        pub fn add_mutation<T>(mut self) -> Self
        where
            T: 'static + BridgedMutation,
        {
            let resolver = Arc::new(|input: &[u8]| {
                let input = match bincode::deserialize::<T::Input>(input) {
                    Ok(m) => m,
                    Err(e) => return async move { Err(BridgeError::Encoding(e)) }.boxed_local(),
                };

                async move {
                    let result = T::resolve(&input).await;
                    bincode::serialize(&result.as_deref()).map_err(BridgeError::Encoding)
                }
                .boxed_local()
            });

            self.resolvers.push(resolver);

            self
        }
    }
}
#[cfg(feature = "resolvable")]
pub use feat_resolvable::*;

#[cfg(not(feature = "resolvable"))]
mod not_feat_resolvable {
    use std::any::TypeId;
    use std::rc::Rc;

    use gloo_net::http::Request;
    use js_sys::Uint8Array;

    use super::*;
    pub(super) use crate::types::{BridgedMutation, BridgedQuery};

    #[derive(Default)]
    pub struct BridgeBuilder {
        query_index: Vec<TypeId>,
    }

    impl Bridge {
        async fn resolve_encoded(&self, type_id: TypeId, input: &[u8]) -> BridgeResult<Vec<u8>> {
            let idx = self
                .inner
                .query_index
                .iter()
                .enumerate()
                .find(|(_, m)| **m == type_id)
                .ok_or(BridgeError::InvalidType(type_id))?
                .0;

            let input = Uint8Array::from(input);
            let resp = Request::post("/_bridge")
                .header("X-Bridge-Type-Idx", &idx.to_string())
                .header("Content-Type", "application/x-bincode")
                .body(input)
                .send()
                .await?;

            resp.binary().await.map_err(|m| m.into())
        }

        pub async fn resolve_query<T>(&self, input: &T::Input) -> QueryResult<T>
        where
            T: 'static + BridgedQuery,
        {
            let inner = move || async move {
                let input = bincode::serialize(&input).map_err(BridgeError::Encoding)?;
                let type_id = TypeId::of::<T>();

                let output = self.resolve_encoded(type_id, &input).await?;
                bincode::deserialize::<std::result::Result<T, T::Error>>(&output)
                    .map_err(BridgeError::Encoding)
            };

            inner().await.map_err(T::into_query_error)?.map(Rc::new)
        }

        pub async fn resolve_mutation<T>(&self, input: &T::Input) -> MutationResult<T>
        where
            T: 'static + BridgedMutation,
        {
            let inner = move || async move {
                let input = bincode::serialize(&input).map_err(BridgeError::Encoding)?;
                let type_id = TypeId::of::<T>();

                let output = self.resolve_encoded(type_id, &input).await?;
                bincode::deserialize::<std::result::Result<T, T::Error>>(&output)
                    .map_err(BridgeError::Encoding)
            };

            inner().await.map_err(T::into_mutation_error)?.map(Rc::new)
        }
    }

    impl BridgeBuilder {
        pub fn add_query<T>(mut self) -> Self
        where
            T: 'static + BridgedQuery,
        {
            let type_id = TypeId::of::<T>();
            self.query_index.push(type_id);

            self
        }

        pub fn add_mutation<T>(mut self) -> Self
        where
            T: 'static + BridgedMutation,
        {
            let type_id = TypeId::of::<T>();
            self.query_index.push(type_id);

            self
        }
    }
}
#[cfg(not(feature = "resolvable"))]
pub use not_feat_resolvable::*;

impl BridgeBuilder {
    pub fn build(self) -> Bridge {
        static ID: AtomicUsize = AtomicUsize::new(0);
        let id = ID.fetch_add(1, Ordering::AcqRel);

        Bridge {
            inner: self.into(),
            id,
        }
    }
}

impl fmt::Debug for BridgeBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BridgeBuilder")
    }
}

pub struct Bridge {
    inner: Arc<BridgeBuilder>,
    id: usize,
}

impl fmt::Debug for Bridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bridge")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Bridge {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Bridge {}

impl Hash for Bridge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.id);
    }
}

impl Bridge {
    pub fn builder() -> BridgeBuilder {
        BridgeBuilder::default()
    }
}

impl Default for Bridge {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Clone for Bridge {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id: self.id,
        }
    }
}
