use anymap2::AnyMap;
use bounce::helmet::HelmetBridge;
use bounce::BounceRoot;
use stellation_bridge::links::Link;
use stellation_bridge::state::BridgeState;
use yew::prelude::*;
use yew_router::BrowserRouter;

#[derive(Properties)]
pub(crate) struct StellationRootProps<L>
where
    L: Link,
{
    #[prop_or_default]
    pub children: Html,
    pub bridge_state: Option<BridgeState<L>>,
}

impl<L> PartialEq for StellationRootProps<L>
where
    L: Link,
{
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children && self.bridge_state == other.bridge_state
    }
}

impl<L> Clone for StellationRootProps<L>
where
    L: Link,
{
    fn clone(&self) -> Self {
        Self {
            children: self.children.clone(),
            bridge_state: self.bridge_state.clone(),
        }
    }
}

#[function_component]
pub(crate) fn StellationRoot<L>(props: &StellationRootProps<L>) -> Html
where
    L: 'static + Link,
{
    let StellationRootProps {
        children,
        bridge_state,
    } = props.clone();

    let get_init_states = use_callback(
        move |_, bridge_state| {
            let mut states = AnyMap::new();

            if let Some(m) = bridge_state.clone() {
                states.insert(m);
            }

            states
        },
        bridge_state,
    );

    html! {
        <BounceRoot {get_init_states}>
            <HelmetBridge />
            <BrowserRouter>
                {children}
            </BrowserRouter>
        </BounceRoot>
    }
}
