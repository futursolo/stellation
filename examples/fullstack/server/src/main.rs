use example_fullstack_api::create_bridge;
use stackable_backend::{Cli, Endpoint};
use tracing::Level;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod app;
use app::ServerApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .with_env_var("STACKABLE_APP_SERVER_LOG")
                .from_env_lossy(),
        )
        .init();

    Cli::builder()
        .endpoint(Endpoint::<ServerApp>::new().with_bridge(create_bridge()))
        .build()
        .run()
        .await?;

    Ok(())
}
