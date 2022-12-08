use std::borrow::Cow;

use bounce::helmet::{HelmetBridge, StaticWriter};
use bounce::{use_atom_setter, BounceRoot};
use stackable_bridge::state::BridgeState;
use stackable_bridge::Bridge;
use yew::prelude::*;
use yew_router::history::{AnyHistory, History, MemoryHistory};
use yew_router::Router;

use crate::ServerAppProps;

#[derive(Properties)]
pub struct StackableRootProps<CTX> {
    pub helmet_writer: StaticWriter,
    pub server_app_props: ServerAppProps<CTX>,
    pub bridge: Bridge,
}

impl<CTX> PartialEq for StackableRootProps<CTX> {
    fn eq(&self, other: &Self) -> bool {
        self.helmet_writer == other.helmet_writer
            && self.server_app_props == other.server_app_props
            && self.bridge == other.bridge
    }
}

impl<CTX> Clone for StackableRootProps<CTX> {
    fn clone(&self) -> Self {
        Self {
            helmet_writer: self.helmet_writer.clone(),
            server_app_props: self.server_app_props.clone(),
            bridge: self.bridge.clone(),
        }
    }
}

#[function_component]
fn Inner<COMP, CTX>(props: &StackableRootProps<CTX>) -> Html
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX>>,
{
    let StackableRootProps {
        helmet_writer,
        server_app_props,
        bridge,
        ..
    } = props.clone();

    let history: AnyHistory = MemoryHistory::new().into();
    history
        .push_with_query(
            server_app_props.path(),
            server_app_props
                .queries::<Vec<(Cow<'_, str>, Cow<'_, str>)>>()
                .expect("failed to parse queries"),
        )
        .expect("failed to push path.");

    let set_bridge = use_atom_setter::<BridgeState>();
    use_memo(
        move |_| {
            set_bridge(BridgeState { inner: bridge });
        },
        (),
    );

    let children = html! { <COMP ..server_app_props /> };

    html! {
        <Router {history}>
            <HelmetBridge writer={helmet_writer} />
            {children}
        </Router>
    }
}

#[function_component]
pub fn StackableRoot<COMP, CTX>(props: &StackableRootProps<CTX>) -> Html
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX>>,
    CTX: 'static,
{
    let props = props.clone();

    html! {
        <BounceRoot>
            <Inner<COMP, CTX> ..props />
        </BounceRoot>
    }
}
