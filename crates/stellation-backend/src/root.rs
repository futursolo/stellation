use std::borrow::Cow;

use anymap2::AnyMap;
use bounce::helmet::{HelmetBridge, StaticWriter};
use bounce::BounceRoot;
use stellation_bridge::links::Link;
use stellation_bridge::state::BridgeState;
use stellation_bridge::Bridge;
use yew::prelude::*;
use yew_router::history::{AnyHistory, History, MemoryHistory};
use yew_router::Router;

use crate::props::ServerAppProps;
use crate::Request;

#[derive(Properties)]
pub(crate) struct StellationRootProps<CTX, REQ, L>
where
    L: Link,
{
    pub helmet_writer: StaticWriter,
    pub server_app_props: ServerAppProps<CTX, REQ>,
    pub bridge: Option<Bridge<L>>,
}

impl<CTX, REQ, L> PartialEq for StellationRootProps<CTX, REQ, L>
where
    L: Link,
{
    fn eq(&self, other: &Self) -> bool {
        self.helmet_writer == other.helmet_writer
            && self.server_app_props == other.server_app_props
            && self.bridge == other.bridge
    }
}

impl<CTX, REQ, L> Clone for StellationRootProps<CTX, REQ, L>
where
    L: Link,
{
    fn clone(&self) -> Self {
        Self {
            helmet_writer: self.helmet_writer.clone(),
            server_app_props: self.server_app_props.clone(),
            bridge: self.bridge.clone(),
        }
    }
}

#[function_component]
pub(crate) fn StellationRoot<COMP, CTX, REQ, L>(props: &StellationRootProps<CTX, REQ, L>) -> Html
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX, REQ>>,
    REQ: 'static + Request<Context = CTX>,
    CTX: 'static,
    L: 'static + Link,
{
    let StellationRootProps {
        helmet_writer,
        server_app_props,
        bridge,
        ..
    } = props.clone();

    let get_init_states = use_callback(
        move |_, bridge| {
            let mut states = AnyMap::new();
            if let Some(m) = bridge.clone().map(BridgeState::from_bridge) {
                states.insert(m);
            }

            states
        },
        bridge,
    );

    let history: AnyHistory = MemoryHistory::new().into();
    history
        .push_with_query(
            server_app_props.path(),
            server_app_props
                .queries::<Vec<(Cow<'_, str>, Cow<'_, str>)>>()
                .expect("failed to parse queries"),
        )
        .expect("failed to push path.");

    let children = html! { <COMP ..server_app_props /> };

    html! {
        <BounceRoot {get_init_states}>
            <HelmetBridge writer={helmet_writer} />
            <Router {history}>
                {children}
            </Router>
        </BounceRoot>
    }
}
