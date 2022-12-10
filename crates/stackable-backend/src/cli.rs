use std::env;
use std::net::ToSocketAddrs;

use anyhow::{anyhow, Context};
use stackable_core::dev::StackctlMetadata;
use typed_builder::TypedBuilder;
use yew::BaseComponent;

use crate::endpoint::Endpoint;
use crate::props::ServerAppProps;
use crate::server::Server;
use crate::Frontend;

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
    pub async fn run(self) -> anyhow::Result<()> {
        let Self { mut endpoint } = self;

        let meta = match env::var(StackctlMetadata::ENV_NAME) {
            Ok(m) => Some(StackctlMetadata::from_json(&m).context("failed to load metadata")?),
            Err(_) => None,
        };

        let addr = meta
            .as_ref()
            .map(|m| m.listen_addr.as_str())
            .unwrap_or_else(|| "localhost:5000");

        if let Some(ref meta) = meta {
            endpoint = endpoint.with_frontend(Frontend::new_path(&meta.frontend_dev_build_dir));
        }

        Server::<()>::bind(
            addr.to_socket_addrs()
                .context("failed to parse address")
                .and_then(|m| {
                    m.into_iter()
                        .next()
                        .ok_or_else(|| anyhow!("failed to parse address"))
                })?,
        )
        .serve_service(endpoint.into_tower_service())
        .await?;

        Ok(())
    }
}
