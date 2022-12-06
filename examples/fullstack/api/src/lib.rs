mod types;
pub use types::*;

#[cfg(feature = "resolvable")]
mod resolvers;

use stackable_bridge::Bridge;

pub fn create_bridge() -> Bridge {
    let mut b = Bridge::new();
    b.add_query::<ServerTimeQuery>();

    b
}
