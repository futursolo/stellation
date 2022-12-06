use std::any::TypeId;
use std::rc::Rc;

use gloo_net::http::Request;
use js_sys::Uint8Array;

use crate::types::{Mutation, MutationResult, Query, QueryResult};

#[derive(Default)]
pub struct RemoteBridge {
    query_index: Vec<TypeId>,
}

impl RemoteBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_query<T>(&mut self) -> &mut Self
    where
        T: 'static + Query,
    {
        let type_id = TypeId::of::<T>();
        self.query_index.push(type_id);

        self
    }

    pub fn add_mutation<T>(&mut self) -> &mut Self
    where
        T: 'static + Mutation,
    {
        let type_id = TypeId::of::<T>();
        self.query_index.push(type_id);

        self
    }

    async fn resolve_encoded(&self, type_id: TypeId, input: &[u8]) -> Vec<u8> {
        let idx = self
            .query_index
            .iter()
            .enumerate()
            .find(|(_, m)| **m == type_id)
            .expect("failed to find query")
            .0;

        let input = Uint8Array::from(input);
        let resp = Request::post("/_bridge")
            .header("X-Bridge-Type-Idx", &idx.to_string())
            .header("Content-Type", "application/x-bincode")
            .body(input)
            .send()
            .await
            .expect("failed to communicate with remote server.");

        resp.binary().await.expect("failed to read body")
    }

    pub async fn resolve_query<T>(&self, input: &T::Input) -> QueryResult<T>
    where
        T: 'static + Query,
    {
        let input = bincode::serialize(&input).expect("failed to serialize");
        let type_id = TypeId::of::<T>();

        let output = self.resolve_encoded(type_id, &input).await;
        bincode::deserialize::<std::result::Result<T, T::Error>>(&output)
            .expect("failed to deserialize.")
            .map(Rc::new)
    }

    pub async fn resolve_mutation<T>(&self, input: &T::Input) -> MutationResult<T>
    where
        T: 'static + Mutation,
    {
        let input = bincode::serialize(&input).expect("failed to serialize");
        let type_id = TypeId::of::<T>();

        let output = self.resolve_encoded(type_id, &input).await;
        bincode::deserialize::<std::result::Result<T, T::Error>>(&output)
            .expect("failed to deserialize.")
            .map(Rc::new)
    }
}
