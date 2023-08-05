use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

use bounce::query::{use_prepared_query, QueryState, UseQueryHandle};
use yew::prelude::*;
use yew::suspense::SuspensionResult;

use super::use_bridged_query_value::BridgedQueryInner;
use crate::links::Link;
use crate::routines::{BridgedQuery, QueryResult};

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

/// A handle returned by [`use_bridged_query`].
///
/// This type dereferences to [`QueryResult<T>`].
pub struct UseBridgedQueryHandle<T, L>
where
    T: BridgedQuery + 'static,
    L: 'static + Link,
{
    inner: UseQueryHandle<BridgedQueryInner<T, L>>,
    state: Rc<BridgedQueryState<T>>,
}

impl<T, L> UseBridgedQueryHandle<T, L>
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

impl<T, L> Clone for UseBridgedQueryHandle<T, L>
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

impl<T, L> Deref for UseBridgedQueryHandle<T, L>
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

impl<T, L> fmt::Debug for UseBridgedQueryHandle<T, L>
where
    T: BridgedQuery + fmt::Debug + 'static,
    L: 'static + Link,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseBridgedQueryHandle")
            .field("state", self.state())
            .finish()
    }
}

/// Bridges a query.
#[hook]
pub fn use_bridged_query<Q, L>(input: Rc<Q::Input>) -> SuspensionResult<UseBridgedQueryHandle<Q, L>>
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

    Ok(UseBridgedQueryHandle {
        inner: handle,
        state,
    })
}
