use std::env;
use std::net::ToSocketAddrs;

use anyhow::{anyhow, Context};
use typed_builder::TypedBuilder;
use yew::BaseComponent;

use crate::{Endpoint, Server};

#[derive(Debug, TypedBuilder)]
pub struct Cli<COMP, F>
where
    COMP: BaseComponent,
{
    endpoint: Endpoint<COMP, F>,
}

impl<COMP, F> Cli<COMP, F>
where
    COMP: BaseComponent,
{
    pub async fn run(self) -> anyhow::Result<()>
    where
        F: 'static + Clone + Send + Fn() -> COMP::Properties,
    {
        let Self { endpoint } = self;

        let addr = env::var("STACKCTL_LISTEN_ADDR")
            .context("starting backend without development server is not yet implemented!")?;
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
