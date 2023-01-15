use std::borrow::Cow;
use std::rc::Rc;

use bounce::helmet::{HelmetBridge, StaticWriter};
use bounce::{use_atom_setter, BounceRoot};
use stellation_bridge::state::{BridgeMetadataState, BridgeState};
use stellation_bridge::{Bridge, BridgeMetadata};
use yew::prelude::*;
use yew_router::history::{AnyHistory, History, MemoryHistory};
use yew_router::Router;

use crate::props::ServerAppProps;

#[derive(Properties)]
pub(crate) struct StellationRootProps<CTX, BCTX> {
    pub helmet_writer: StaticWriter,
    pub server_app_props: ServerAppProps<CTX>,
    pub bridge: Bridge,
    pub bridge_metadata: Rc<BridgeMetadata<BCTX>>,
}

impl<CTX, BCTX> PartialEq for StellationRootProps<CTX, BCTX> {
    fn eq(&self, other: &Self) -> bool {
        self.helmet_writer == other.helmet_writer
            && self.server_app_props == other.server_app_props
            && self.bridge == other.bridge
            && Rc::ptr_eq(&self.bridge_metadata, &other.bridge_metadata)
    }
}

impl<CTX, BCTX> Clone for StellationRootProps<CTX, BCTX> {
    fn clone(&self) -> Self {
        Self {
            helmet_writer: self.helmet_writer.clone(),
            server_app_props: self.server_app_props.clone(),
            bridge: self.bridge.clone(),
            bridge_metadata: self.bridge_metadata.clone(),
        }
    }
}

#[function_component]
fn Inner<COMP, CTX, BCTX>(props: &StellationRootProps<CTX, BCTX>) -> Html
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX>>,
    BCTX: 'static,
{
    let StellationRootProps {
        helmet_writer,
        server_app_props,
        bridge,
        bridge_metadata,
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
    let set_bridge_metadata = use_atom_setter::<BridgeMetadataState<BCTX>>();

    use_memo(
        move |_| {
            set_bridge(BridgeState { inner: bridge });
            set_bridge_metadata(BridgeMetadataState::from(bridge_metadata));
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
pub(crate) fn StellationRoot<COMP, CTX, BCTX>(props: &StellationRootProps<CTX, BCTX>) -> Html
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX>>,
    CTX: 'static,
    BCTX: 'static,
{
    let props = props.clone();

    html! {
        <BounceRoot>
            <Inner<COMP, CTX, BCTX> ..props />
        </BounceRoot>
    }
}
