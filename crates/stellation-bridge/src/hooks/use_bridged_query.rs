use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::{use_prepared_query, UseQueryHandle};
use bounce::BounceStates;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use yew::prelude::*;
use yew::suspense::SuspensionResult;

use crate::links::Link;
use crate::routines::{BridgedQuery, QueryResult};
use crate::state::BridgeState;

#[derive(Debug)]
struct QueryState<Q, L>
where
    Q: BridgedQuery,
{
    inner: QueryResult<Q>,
    _marker: PhantomData<L>,
}

impl<Q, L> Clone for QueryState<Q, L>
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

impl<Q, L> PartialEq for QueryState<Q, L>
where
    Q: BridgedQuery + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<Q, L> Eq for QueryState<Q, L> where Q: BridgedQuery + Eq {}

impl<Q, L> Serialize for QueryState<Q, L>
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

impl<'de, Q, L> Deserialize<'de> for QueryState<Q, L>
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
impl<Q, L> bounce::query::Query for QueryState<Q, L>
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
        let bridge = states.get_atom_value::<BridgeState<L>>();

        let bridge = bridge.inner.as_ref().expect("bridge is not set?");
        let token = bridge.read_token(states);

        let link = match token {
            Some(m) => Cow::Owned(bridge.link.with_token(m.as_ref())),
            None => Cow::Borrowed(&bridge.link),
        };

        Ok(Self {
            inner: link.resolve_query::<Q>(&input).await,
            _marker: PhantomData,
        }
        .into())
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
    inner: UseQueryHandle<QueryState<T, L>>,
}

impl<T, L> UseBridgedQueryHandle<T, L>
where
    T: BridgedQuery + 'static,
    L: 'static + Link,
{
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
        match self.inner.deref() {
            Ok(ref m) => &m.inner,
            _ => panic!("this variant can never happen!"),
        }
    }
}

impl<T, L> fmt::Debug for UseBridgedQueryHandle<T, L>
where
    T: BridgedQuery + fmt::Debug + 'static,
    L: 'static + Link,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UseQueryHandle")
            .field("value", self.deref())
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
    let handle = use_prepared_query::<QueryState<Q, L>>(input)?;

    Ok(UseBridgedQueryHandle { inner: handle })
}
