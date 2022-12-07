use std::net::ToSocketAddrs;

use anyhow::{anyhow, Context};
use typed_builder::TypedBuilder;
use yew::BaseComponent;

use crate::dev_env::DevEnv;
use crate::{Endpoint, Server, ServerAppProps};

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
        let dev_env = DevEnv::from_env()?
            .context("starting backend without development server is not yet implemented!")?;

        endpoint.set_dev_env(dev_env.clone());

        let addr = dev_env.listen_addr;
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
