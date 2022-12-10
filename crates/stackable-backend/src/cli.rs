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
        let Self { endpoint } = self;
        let meta = env::var(StackctlMetadata::ENV_NAME)
            .map(|m| StackctlMetadata::from_json(&m))
            .context("starting backend without development server is not yet implemented!")?
            .context("failed to load metadata")?;

        let addr = meta.listen_addr;
        Server::<()>::bind(
            addr.to_socket_addrs()
                .context("failed to parse address")
                .and_then(|m| {
                    m.into_iter()
                        .next()
                        .ok_or_else(|| anyhow!("failed to parse address"))
                })?,
        )
        .serve_service(
            endpoint
                .with_frontend(Frontend::new_path(meta.frontend_dev_build_dir))
                .into_tower_service(),
        )
        .await?;

        Ok(())
    }
}
