use std::env;
use std::net::ToSocketAddrs;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use clap::Parser;
use stellation_core::dev::StctlMetadata;
use typed_builder::TypedBuilder;
use yew::BaseComponent;

use crate::endpoint::Endpoint;
use crate::props::ServerAppProps;
use crate::server::Server;
use crate::Frontend;

#[derive(Parser)]
struct Arguments {
    /// The address to listen to.
    #[arg(long, default_value = "localhost:5000", env = "STELLATION_LISTEN_ADDR")]
    listen_addr: String,
    /// The ditectory that contains the frontend artifact.
    #[arg(long, env = "STELLATION_FRONTEND_DIR")]
    frontend_dir: Option<PathBuf>,
}

/// The default command line instance for the backend server.
#[derive(Debug, TypedBuilder)]
pub struct Cli<COMP, CTX = ()>
where
    COMP: BaseComponent,
{
    endpoint: Endpoint<COMP, CTX>,
}

impl<COMP, CTX> Cli<COMP, CTX>
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX>>,
    CTX: 'static,
{
    /// Parses the arguments and runs the server.
    pub async fn run(self) -> anyhow::Result<()> {
        let Self { mut endpoint } = self;

        let args = Arguments::parse();

        // Prioritise information from stctl.
        let meta = match env::var(StctlMetadata::ENV_NAME) {
            Ok(m) => Some(StctlMetadata::from_json(&m).context("failed to load metadata")?),
            Err(_) => None,
        };

        let addr = meta
            .as_ref()
            .map(|m| m.listen_addr.as_str())
            .unwrap_or_else(|| args.listen_addr.as_str());

        if let Some(ref p) = args.frontend_dir {
            endpoint = endpoint.with_frontend(Frontend::new_path(p));
        }

        if let Some(ref meta) = meta {
            endpoint = endpoint
                .with_frontend(Frontend::new_path(&meta.frontend_dev_build_dir))
                .with_auto_refresh();
        }

        let listen_addr = addr
            .to_socket_addrs()
            .context("failed to parse address")
            .and_then(|m| {
                m.into_iter()
                    .next()
                    .ok_or_else(|| anyhow!("failed to parse address"))
            })?;

        tracing::info!("Listening at: http://{}/", addr);

        Server::<()>::bind(listen_addr)
            .serve_service(endpoint.into_tower_service())
            .await?;

        Ok(())
    }
}
