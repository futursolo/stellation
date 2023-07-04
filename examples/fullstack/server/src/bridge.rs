use example_fullstack_api::{
    create_resolver_registry, create_routine_registry, DefaultBridge, DefaultLink,
};
use stellation_backend_tower::TowerRequest;

pub async fn create_backend_bridge(_req: TowerRequest<()>) -> DefaultBridge {
    DefaultBridge::new(
        DefaultLink::builder()
            .context(())
            .resolvers(create_resolver_registry())
            .routines(create_routine_registry())
            .build(),
    )
}
