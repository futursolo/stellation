use async_trait::async_trait;
use stellation_bridge::links::LocalLink;
use stellation_bridge::registry::ResolverRegistry;
use stellation_bridge::resolvers::{MutationResolver, QueryResolver};
use stellation_bridge::types::{MutationResult, QueryResult};
use stellation_bridge::Bridge;
use time::OffsetDateTime;

pub use crate::types::*;

#[async_trait(?Send)]
impl QueryResolver for ServerTimeQuery {
    type Context = ();

    async fn resolve(_metadata: &(), _input: &Self::Input) -> QueryResult<Self> {
        Ok(Self {
            value: OffsetDateTime::now_utc(),
        }
        .into())
    }
}

#[async_trait(?Send)]
impl MutationResolver for GreetingMutation {
    type Context = ();

    async fn resolve(_metadata: &(), name: &Self::Input) -> MutationResult<Self> {
        Ok(Self {
            message: format!("Hello, {name}!"),
        }
        .into())
    }
}

pub fn create_resolver_registry() -> ResolverRegistry<()> {
    ResolverRegistry::<()>::builder()
        .add_query::<ServerTimeQuery>()
        .add_mutation::<GreetingMutation>()
        .build()
}

pub type DefaultLink = LocalLink<()>;
pub type DefaultBridge = Bridge<DefaultLink>;

pub async fn create_backend_bridge(_req: Option<String>) -> DefaultBridge {
    Bridge::new(
        LocalLink::builder()
            .context(())
            .resolvers(create_resolver_registry())
            .routines(create_routine_registry())
            .build(),
    )
}
