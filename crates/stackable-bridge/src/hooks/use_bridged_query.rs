use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_prepared_query, UseQueryHandle};
use bounce::BounceStates;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use yew::prelude::*;
use yew::suspense::SuspensionResult;

#[cfg(feature = "resolvable")]
use crate::resolvers::QueryResolver as BridgedQuery;
use crate::state::BridgeState;
#[cfg(not(feature = "resolvable"))]
use crate::types::BridgedQuery;
use crate::types::QueryResult;

#[derive(Debug, PartialEq, Eq)]
struct QueryState<Q> {
    inner: Rc<Q>,
}

impl<Q> Clone for QueryState<Q> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Q> Serialize for QueryState<Q>
where
    Q: 'static + BridgedQuery + PartialEq,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (*self.inner).serialize(serializer)
    }
}

impl<'de, Q> Deserialize<'de> for QueryState<Q>
where
    Q: 'static + BridgedQuery + PartialEq,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Q::deserialize(deserializer)
            .map(Rc::new)
            .map(|inner| Self { inner })
    }
}
#[async_trait(?Send)]
impl<Q> bounce::query::Query for QueryState<Q>
where
    Q: 'static + BridgedQuery + PartialEq,
{
    type Error = Q::Error;
    type Input = Q::Input;

    async fn query(
        states: &BounceStates,
        input: Rc<Self::Input>,
    ) -> bounce::query::QueryResult<Self> {
        let bridge = states.get_atom_value::<BridgeState>();

        bridge
            .inner
            .resolve_query::<Q>(&input)
            .await
            .map(|inner| Self { inner })
            .map(Rc::new)
    }
}

/// A handle returned by [`use_bridged_query`].
pub struct UseBridgedQueryHandle<T>
where
    T: BridgedQuery + 'static,
{
    inner: UseQueryHandle<QueryState<T>>,
    result: QueryResult<T>,
}

impl<T> UseBridgedQueryHandle<T>
where
    T: BridgedQuery + 'static,
{
    /// Refreshes the query.
    ///
    /// The query will be refreshed with the input provided to the hook.
    pub async fn refresh(&self) -> QueryResult<T> {
        self.inner
            .refresh()
            .await
            .as_deref()
            .map(|m| m.inner.clone())
            .map_err(|e| e.clone())
    }
}

impl<T> Clone for UseBridgedQueryHandle<T>
where
    T: BridgedQuery + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            result: self.result.clone(),
        }
    }
}

impl<T> Deref for UseBridgedQueryHandle<T>
where
    T: BridgedQuery + 'static,
{
    type Target = QueryResult<T>;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

impl<T> fmt::Debug for UseBridgedQueryHandle<T>
where
    T: BridgedQuery + fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseQueryHandle")
            .field("value", &self.result)
            .finish()
    }
}

#[hook]
pub fn use_bridged_query<Q>(input: Rc<Q::Input>) -> SuspensionResult<UseBridgedQueryHandle<Q>>
where
    Q: 'static + BridgedQuery,
{
    let handle = use_prepared_query::<QueryState<Q>>(input)?;
    let result = handle
        .as_deref()
        .map(|m| m.inner.clone())
        .map_err(|e| e.clone());

    Ok(UseBridgedQueryHandle {
        inner: handle,
        result,
    })
}
