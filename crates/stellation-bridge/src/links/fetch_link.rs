use std::cell::Cell;

use async_trait::async_trait;
use futures::{future, FutureExt, TryFutureExt};
use gloo_net::http::Request;
use js_sys::Uint8Array;
use typed_builder::TypedBuilder;

use super::Link;
use crate::registry::RoutineRegistry;
use crate::routines::{BridgedMutation, BridgedQuery, MutationResult, QueryResult};
use crate::{BridgeError, BridgeResult};

/// A Link implemented with `fetch`, this requires a WebAssembly target with available global
/// `fetch`.
///
/// # Example
///
/// ```
/// # use crate::links::FetchLink;
/// # use crate::registry::RoutineRegistry;
/// # let routines = RoutineRegistry::builder().build();
/// let link = FetchLink::builder()
///     .url("/_bridge") // Defaults to `/_bridge`, which is also default on most first party implementations.
///     .routines(routines)
///     .build();
/// ```
#[derive(TypedBuilder, Debug, Clone)]
pub struct FetchLink {
    /// The bridge URL, defaults to `/_bridge`, which is also the default used by official backend
    /// implementations.
    #[builder(setter(into), default_code = r#""/_bridge".to_string()"#)]
    url: String,
    /// The routine registry for all registered routines.
    routines: RoutineRegistry,
    /// The bearer token to send to the server.
    #[builder(setter(skip), default)]
    token: Option<String>,

    /// The link equity tracker.
    #[builder(setter(skip), default_code = r#"FetchLink::next_id()"#)]
    id: usize,
}

impl PartialEq for FetchLink {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl FetchLink {
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
impl Link for FetchLink {
    fn with_token<T>(&self, token: T) -> Self
    where
        T: AsRef<str>,
    {
        let mut self_ = self.clone();

        self_.token = Some(token.as_ref().to_string());

        self_
    }

    async fn resolve_encoded(&self, input_buf: &[u8]) -> BridgeResult<Vec<u8>> {
        future::ready(self.url.as_str())
            .map(Request::post)
            .map(|m| m.header("content-type", "application/x-bincode"))
            .map(|req| {
                if let Some(ref m) = self.token {
                    return req.header("authorization", &format!("Bearer {m}"));
                }

                req
            })
            .map(move |m| m.body(&Uint8Array::from(input_buf)))
            .and_then(|m| m.send())
            .and_then(|m| async move { m.binary().await })
            .map_err(BridgeError::Network)
            .await
    }

    async fn resolve_query<T>(&self, input: &T::Input) -> QueryResult<T>
    where
        T: 'static + BridgedQuery,
    {
        future::ready(input)
            .map(|m| self.routines.encode_query_input::<T>(m))
            .and_then(|m| async move { self.resolve_encoded(&m).await })
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
            .and_then(|m| async move { self.resolve_encoded(&m).await })
            .map_err(T::into_mutation_error)
            .and_then(|m| async move { self.routines.decode_mutation_output::<T>(&m) })
            .await
    }
}
