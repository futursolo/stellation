#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

mod types;
pub use types::*;

#[cfg(feature = "resolvable")]
mod resolvers;

#[cfg(feature = "resolvable")]
pub use resolvers::{create_resolver_registry, DefaultLink};
use stellation_bridge::registry::RoutineRegistry;
use stellation_bridge::Bridge;
#[cfg(not(feature = "resolvable"))]
pub use types::DefaultLink;

pub type DefaultBridge = Bridge<DefaultLink>;

pub fn create_routine_registry() -> RoutineRegistry {
    RoutineRegistry::builder()
        .add_query::<ServerTimeQuery>()
        .add_mutation::<GreetingMutation>()
        .build()
}
