#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

use example_fullstack_api::create_bridge;
use stellation_backend::Endpoint;
use stellation_backend_cli::Cli;

mod app;
use app::ServerApp;

#[cfg(stellation_embedded_frontend)]
#[derive(rust_embed::RustEmbed)]
#[folder = "$STELLATION_FRONTEND_BUILD_DIR"]
struct Frontend;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    stellation_backend_cli::trace::init_default("STELLATION_APP_SERVER_LOG");

    let endpoint = Endpoint::<ServerApp>::new().with_bridge(create_bridge());

    #[cfg(stellation_embedded_frontend)]
    let endpoint = endpoint.with_frontend(stellation_backend::Frontend::new_embedded::<Frontend>());

    Cli::builder().endpoint(endpoint).build().run().await?;

    Ok(())
}
