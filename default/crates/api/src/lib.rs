#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

mod types;
pub use types::*;

#[cfg(feature = "resolvable")]
mod resolvers;

use stellation_bridge::Bridge;

pub fn create_bridge() -> Bridge {
    Bridge::builder()
        .add_query::<ServerTimeQuery>()
        .add_mutation::<GreetingMutation>()
        .build()
}
