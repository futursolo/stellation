use bounce::query::use_prepared_query as use_base_prepared_query;
use yew::prelude::*;

#[cfg(feature = "resolvable")]
use crate::resolvers::QueryResolver;
use crate::types::{Query, QueryResult};
use crate::Bridge;

#[cfg(feature = "resolvable")]
#[hook]
pub fn use_prepared_query<Q>() -> QueryResult<Q>
where
    Q: Query + QueryResolver,
{
    use std::rc::Rc;

    use async_trait::async_trait;
    use bounce::BounceStates;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq)]
    pub struct QueryState<Q>
    where
        Q: 'static + Query + QueryResolver + Serialize + for<'de> Deserialize<'de> + PartialEq,
    {
        inner: Q,
    }

    #[async_trait(?Send)]
    impl<Q> bounce::query::Query for QueryState<Q>
    where
        Q: 'static + Query + QueryResolver + Serialize + for<'de> Deserialize<'de> + PartialEq,
    {
        type Error = Q::Error;
        type Input = Q::Input;

        async fn query(
            states: &BounceStates,
            input: Rc<Self::Input>,
        ) -> bounce::query::QueryResult<Self> {
            todo!()
        }
    }

    let bridge = use_context::<Bridge>();

    todo!()
}

#[cfg(not(feature = "resolvable"))]
#[hook]
pub fn use_prepared_query<Q>() -> QueryResult<Q>
where
    Q: Query,
{
    todo!()
}
