use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use futures::future::LocalBoxFuture;
use futures::FutureExt;

use crate::resolvers::{MutationResolver, QueryResolver};
use crate::types::{Mutation, Query, QueryResult};

type Resolvers = HashMap<TypeId, Arc<dyn Fn(&[u8]) -> LocalBoxFuture<'static, Vec<u8>>>>;

#[derive(Default)]
pub struct Bridge {
    resolvers: Resolvers,
    query_index: Vec<TypeId>,
}

impl Bridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_query<T>(&mut self) -> &mut Self
    where
        T: 'static + Query + QueryResolver,
    {
        let type_id = TypeId::of::<T>();

        let resolver = Arc::new(|input: &[u8]| {
            let input = bincode::deserialize::<T::Input>(input)
                .expect("failed to deserialize, to be implemented");

            async move {
                let result = T::resolve(&input).await;
                bincode::serialize(&result.as_deref())
                    .expect("failed to serialize, to be implemented")
            }
            .boxed_local()
        });

        self.resolvers.insert(type_id, resolver);
        self.query_index.push(type_id);

        self
    }

    pub fn add_mutation<T>(&mut self) -> &mut Self
    where
        T: 'static + Mutation + MutationResolver,
    {
        let type_id = TypeId::of::<T>();

        let resolver = Arc::new(|input: &[u8]| {
            let input = bincode::deserialize::<T::Input>(input)
                .expect("failed to deserialize, to be implemented");

            async move {
                let result = T::resolve(&input).await;
                bincode::serialize(&result.as_deref())
                    .expect("failed to serialize, to be implemented")
            }
            .boxed_local()
        });

        self.resolvers.insert(type_id, resolver);
        self.query_index.push(type_id);

        self
    }

    pub async fn resolve_query<T>(&self, input: &T::Input) -> QueryResult<T>
    where
        T: 'static + Query + QueryResolver,
    {
        T::resolve(input).await
    }

    pub async fn resolve_mutation<T>(&self, input: &T::Input) -> QueryResult<T>
    where
        T: 'static + Query + QueryResolver,
    {
        T::resolve(input).await
    }
}
