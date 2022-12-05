use std::collections::HashMap;

use bounce::helmet::{HelmetBridge, StaticWriter};
use bounce::BounceRoot;
use yew::prelude::*;
use yew_router::history::{AnyHistory, History, MemoryHistory};
use yew_router::Router;

#[derive(Properties, PartialEq, Clone)]
pub struct StackableRootProps {
    #[prop_or_default]
    pub children: Html,
    pub helmet_writer: StaticWriter,
    pub path: String,
    pub queries: HashMap<String, String>,
}

#[function_component]
pub fn StackableRoot(props: &StackableRootProps) -> Html {
    let StackableRootProps {
        children,
        helmet_writer,
        path,
        queries,
    } = props.clone();

    let history: AnyHistory = MemoryHistory::new().into();
    history
        .push_with_query(path, queries)
        .expect("failed to push path.");

    html! {
        <BounceRoot>
            <Router {history}>
                <HelmetBridge writer={helmet_writer} />
                {children}
            </Router>
        </BounceRoot>
    }
}
