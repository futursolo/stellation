#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

use example_fullstack_api::create_bridge;
use stackable_backend::{Cli, Endpoint};

mod app;
use app::ServerApp;

#[cfg(stackable_embedded_frontend)]
#[derive(rust_embed::RustEmbed)]
#[folder = "$STACKABLE_FRONTEND_BUILD_DIR"]
struct Frontend;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    stackable_backend::trace::init_default("STACKABLE_APP_SERVER_LOG");

    let endpoint = Endpoint::<ServerApp>::new().with_bridge(create_bridge());

    #[cfg(stackable_embedded_frontend)]
    let endpoint = endpoint.with_frontend(stackable_backend::Frontend::new_embedded::<Frontend>());

    Cli::builder().endpoint(endpoint).build().run().await?;

    Ok(())
}
