use std::borrow::Cow;
use std::marker::PhantomData;

use bounce::helmet::{HelmetBridge, StaticWriter};
use bounce::BounceRoot;
use yew::prelude::*;
use yew_router::history::{AnyHistory, History, MemoryHistory};
use yew_router::Router;

use crate::ServerAppProps;

#[derive(Properties)]
pub struct StackableRootProps<COMP, CTX> {
    pub helmet_writer: StaticWriter,
    pub server_app_props: ServerAppProps<CTX>,
    #[prop_or_default]
    pub _marker: PhantomData<COMP>,
}

impl<COMP, CTX> PartialEq for StackableRootProps<COMP, CTX> {
    fn eq(&self, other: &Self) -> bool {
        self.helmet_writer == other.helmet_writer && self.server_app_props == other.server_app_props
    }
}

impl<COMP, CTX> Clone for StackableRootProps<COMP, CTX> {
    fn clone(&self) -> Self {
        Self {
            helmet_writer: self.helmet_writer.clone(),
            server_app_props: self.server_app_props.clone(),
            _marker: PhantomData,
        }
    }
}

#[function_component]
pub fn StackableRoot<COMP, CTX>(props: &StackableRootProps<COMP, CTX>) -> Html
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX>>,
{
    let StackableRootProps {
        helmet_writer,
        server_app_props,
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

    html! {
        <BounceRoot>
            <Router {history}>
                <HelmetBridge writer={helmet_writer} />
                <COMP ..server_app_props />
            </Router>
        </BounceRoot>
    }
}
