use std::rc::Rc;

use async_trait::async_trait;
use bounce::query::use_prepared_query;
use bounce::BounceStates;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use yew::prelude::*;
use yew::suspense::SuspensionResult;

#[cfg(feature = "resolvable")]
use crate::resolvers::QueryResolver;
use crate::state::BridgeState;
use crate::types::{BridgedQuery, QueryResult};

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

#[cfg(feature = "resolvable")]
#[hook]
pub fn use_bridged_query<Q>(input: Rc<Q::Input>) -> SuspensionResult<QueryResult<Q>>
where
    Q: 'static + BridgedQuery + QueryResolver,
{
    #[async_trait(?Send)]
    impl<Q> bounce::query::Query for QueryState<Q>
    where
        Q: 'static + BridgedQuery + QueryResolver + PartialEq,
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

    Ok(use_prepared_query::<QueryState<Q>>(input)?
        .as_deref()
        .map(|m| m.inner.clone())
        .map_err(|e| e.clone()))
}

#[cfg(not(feature = "resolvable"))]
#[hook]
pub fn use_bridged_query<Q>(input: Rc<Q::Input>) -> SuspensionResult<QueryResult<Q>>
where
    Q: 'static + BridgedQuery,
{
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

    Ok(use_prepared_query::<QueryState<Q>>(input)?
        .as_deref()
        .map(|m| m.inner.clone())
        .map_err(|e| e.clone()))
}
