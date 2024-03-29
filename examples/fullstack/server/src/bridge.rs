use example_fullstack_api::{create_resolver_registry, create_routine_registry, Bridge, Link};
use stellation_backend_tower::TowerRequest;

pub async fn create_backend_bridge(_req: TowerRequest<()>) -> Bridge {
    Bridge::new(
        Link::builder()
            .context(())
            .resolvers(create_resolver_registry())
            .routines(create_routine_registry())
            .build(),
    )
}
