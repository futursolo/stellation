use example_fullstack_app::ServerApp;
use stackable_backend::{Cli, Endpoint};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    Cli::builder()
        .endpoint(Endpoint::<ServerApp, ()>::new())
        .build()
        .run()
        .await?;

    Ok(())
}
