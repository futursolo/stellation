use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_prepared_query, QueryState, UseQueryHandle};
use bounce::BounceStates;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use yew::prelude::*;
use yew::suspense::SuspensionResult;

use crate::links::Link;
use crate::routines::{BridgedQuery, QueryResult};
use crate::state::BridgeSelector;

/// Bridged Query State
#[derive(Debug, PartialEq)]
pub enum BridgedQueryState<T>
where
    T: BridgedQuery + 'static,
{
    /// The query has completed.
    Completed {
        /// Result of the completed query.
        result: QueryResult<T>,
    },
    /// A previous query has completed and a new query is currently loading.
    Refreshing {
        /// Result of last completed query.
        last_result: QueryResult<T>,
    },
}

impl<T> Clone for BridgedQueryState<T>
where
    T: BridgedQuery + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Completed { result } => Self::Completed {
                result: result.clone(),
            },
            Self::Refreshing { last_result } => Self::Refreshing {
                last_result: last_result.clone(),
            },
        }
    }
}

impl<T> PartialEq<&BridgedQueryState<T>> for BridgedQueryState<T>
where
    T: BridgedQuery + 'static,
{
    fn eq(&self, other: &&BridgedQueryState<T>) -> bool {
        self == *other
    }
}

impl<T> PartialEq<BridgedQueryState<T>> for &'_ BridgedQueryState<T>
where
    T: BridgedQuery + 'static,
{
    fn eq(&self, other: &BridgedQueryState<T>) -> bool {
        *self == other
    }
}

#[derive(Debug)]
pub(super) struct BridgedQueryInner<Q, L>
where
    Q: BridgedQuery,
{
    pub inner: QueryResult<Q>,
    _marker: PhantomData<L>,
}

impl<Q, L> Clone for BridgedQueryInner<Q, L>
where
    Q: BridgedQuery,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Q, L> PartialEq for BridgedQueryInner<Q, L>
where
    Q: BridgedQuery + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<Q, L> Eq for BridgedQueryInner<Q, L> where Q: BridgedQuery + Eq {}

impl<Q, L> Serialize for BridgedQueryInner<Q, L>
where
    Q: BridgedQuery,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.as_deref().serialize(serializer)
    }
}

impl<'de, Q, L> Deserialize<'de> for BridgedQueryInner<Q, L>
where
    Q: BridgedQuery,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            inner: std::result::Result::<Q, Q::Error>::deserialize(deserializer)?.map(Rc::new),
            _marker: PhantomData,
        })
    }
}
#[async_trait(?Send)]
impl<Q, L> bounce::query::Query for BridgedQueryInner<Q, L>
where
    Q: 'static + BridgedQuery,
    L: 'static + Link,
{
    type Error = Q::Error;
    type Input = Q::Input;

    async fn query(
        states: &BounceStates,
        input: Rc<Self::Input>,
    ) -> bounce::query::QueryResult<Self> {
        let bridge = states.get_selector_value::<BridgeSelector<L>>();
        let link = bridge.link();

        Ok(Self {
            inner: link.resolve_query::<Q>(&input).await,
            _marker: PhantomData,
        }
        .into())
    }
}

/// A handle returned by [`use_bridged_query_value`].
pub struct UseBridgedQueryValueHandle<T, L>
where
    T: BridgedQuery + 'static,
    L: 'static + Link,
{
    inner: UseQueryHandle<BridgedQueryInner<T, L>>,
    state: Rc<BridgedQueryState<T>>,
}

impl<T, L> UseBridgedQueryValueHandle<T, L>
where
    T: BridgedQuery + 'static,
    L: 'static + Link,
{
    /// Returns the state of current query.
    pub fn state(&self) -> &BridgedQueryState<T> {
        self.state.as_ref()
    }

    /// Refreshes the query.
    ///
    /// The query will be refreshed with the input provided to the hook.
    pub async fn refresh(&self) -> QueryResult<T> {
        self.inner.refresh().await?.inner.clone()
    }
}

impl<T, L> Clone for UseBridgedQueryValueHandle<T, L>
where
    T: BridgedQuery + 'static,
    L: 'static + Link,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            state: self.state.clone(),
        }
    }
}

impl<T, L> Deref for UseBridgedQueryValueHandle<T, L>
where
    T: BridgedQuery + 'static,
    L: 'static + Link,
{
    type Target = QueryResult<T>;

    fn deref(&self) -> &Self::Target {
        match self.state() {
            BridgedQueryState::Completed { result }
            | BridgedQueryState::Refreshing {
                last_result: result,
            } => result,
        }
    }
}

impl<T, L> fmt::Debug for UseBridgedQueryValueHandle<T, L>
where
    T: BridgedQuery + fmt::Debug + 'static,
    L: 'static + Link,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseBridgedQueryValueHandle")
            .field("state", self.state())
            .finish()
    }
}

/// Bridges a query as value.
///
/// # Note
///
/// This hook does not suspend the component and the data is not fetched during SSR.
/// If this hook is used in SSR, this hook will remain as loading state.
#[hook]
pub fn use_bridged_query_value<Q, L>(
    input: Rc<Q::Input>,
) -> SuspensionResult<UseBridgedQueryValueHandle<Q, L>>
where
    Q: 'static + BridgedQuery,
    L: 'static + Link,
{
    let handle = use_prepared_query::<BridgedQueryInner<Q, L>>(input)?;
    let state = use_memo(
        |state| match state {
            QueryState::Completed { result } => BridgedQueryState::Completed {
                result: result
                    .as_ref()
                    .map_err(|e| e.clone())
                    .and_then(|m| m.inner.clone()),
            },
            QueryState::Refreshing { last_result } => BridgedQueryState::Refreshing {
                last_result: last_result
                    .as_ref()
                    .map_err(|e| e.clone())
                    .and_then(|m| m.inner.clone()),
            },
        },
        handle.state().clone(),
    );

    Ok(UseBridgedQueryValueHandle {
        inner: handle,
        state,
    })
}
