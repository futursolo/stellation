use std::fmt;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::error::{BridgeError, BridgeResult};
use crate::types::{MutationResult, QueryResult};

#[derive(Debug, Serialize, Deserialize)]
struct Incoming<'a> {
    query_index: usize,
    input: &'a [u8],
}

#[cfg(feature = "resolvable")]
mod feat_resolvable {
    use std::rc::Rc;
    use std::sync::Arc;

    use bounce::{BounceStates, Selector};
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
        pub(crate) async fn resolve_query<T>(&self, input: &T::Input) -> QueryResult<T>
        where
            T: 'static + BridgedQuery,
        {
            T::resolve(input).await
        }

        pub(crate) async fn resolve_mutation<T>(&self, input: &T::Input) -> MutationResult<T>
        where
            T: 'static + BridgedMutation,
        {
            T::resolve(input).await
        }

        pub async fn resolve_encoded(&self, incoming: &[u8]) -> BridgeResult<Vec<u8>> {
            let incoming: Incoming<'_> = bincode::deserialize(incoming)?;

            let resolver = self
                .inner
                .resolvers
                .get(incoming.query_index)
                .ok_or(BridgeError::InvalidIndex(incoming.query_index))?;

            resolver(incoming.input).await
        }

        pub(crate) fn read_token(&self, _states: &BounceStates) -> Option<Rc<dyn AsRef<str>>> {
            None
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

        pub fn with_token_selector<T>(self) -> Self
        where
            T: 'static + Selector + AsRef<str>,
        {
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

    use bounce::{BounceStates, Selector};
    use gloo_net::http::{Headers, Request};
    use js_sys::Uint8Array;

    use super::*;
    pub(super) use crate::types::{BridgedMutation, BridgedQuery};

    type ReadToken = Box<dyn Fn(&BounceStates) -> Rc<dyn AsRef<str>>>;

    #[derive(Default)]
    pub struct BridgeBuilder {
        query_ids: Vec<TypeId>,
        read_token: Option<ReadToken>,
    }

    impl Bridge {
        async fn resolve_encoded(&self, type_id: TypeId, input: &[u8]) -> BridgeResult<Vec<u8>> {
            let idx = self
                .inner
                .query_ids
                .iter()
                .enumerate()
                .find(|(_, m)| **m == type_id)
                .ok_or(BridgeError::InvalidType(type_id))?
                .0;

            let incoming = Incoming {
                query_index: idx,
                input,
            };

            let incoming = bincode::serialize(&incoming)?;

            let input = Uint8Array::from(incoming.as_slice());
            let resp = Request::post("/_bridge")
                .header("Content-Type", "application/x-bincode")
                .body(input)
                .send()
                .await?;

            resp.binary().await.map_err(|m| m.into())
        }

        pub(crate) async fn resolve_query<T>(&self, input: &T::Input) -> QueryResult<T>
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

        pub(crate) async fn resolve_mutation<T>(&self, input: &T::Input) -> MutationResult<T>
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

        pub(crate) fn read_token(&self, states: &BounceStates) -> Option<Rc<dyn AsRef<str>>> {
            self.inner.read_token.as_ref().map(|m| m(states))
        }
    }

    impl BridgeBuilder {
        pub fn add_query<T>(mut self) -> Self
        where
            T: 'static + BridgedQuery,
        {
            let type_id = TypeId::of::<T>();
            self.query_ids.push(type_id);

            self
        }

        pub fn add_mutation<T>(mut self) -> Self
        where
            T: 'static + BridgedMutation,
        {
            let type_id = TypeId::of::<T>();
            self.query_ids.push(type_id);

            self
        }

        pub fn with_token_selector<T>(mut self) -> Self
        where
            T: 'static + Selector + AsRef<str>,
        {
            let read_token = Box::new(move |states: &BounceStates| {
                let state = states.get_selector_value::<T>();

                state as Rc<dyn AsRef<str>>
            }) as ReadToken;

            self.read_token = Some(read_token);

            self
        }
    }
}
#[cfg(not(feature = "resolvable"))]
pub use not_feat_resolvable::*;
use serde::{Deserialize, Serialize};

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
