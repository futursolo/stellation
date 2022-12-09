mod types;
pub use types::*;

#[cfg(feature = "resolvable")]
mod resolvers;

use stackable_bridge::Bridge;

pub fn create_bridge() -> Bridge {
    Bridge::builder().add_query::<ServerTimeQuery>().build()
}
