use anymap2::AnyMap;
use bounce::helmet::HelmetBridge;
use bounce::BounceRoot;
use stellation_bridge::links::Link;
use stellation_bridge::state::BridgeState;
use stellation_bridge::Bridge;
use yew::prelude::*;
use yew_router::BrowserRouter;

#[derive(Properties)]
pub(crate) struct StellationRootProps<L> {
    #[prop_or_default]
    pub children: Html,
    pub bridge: Option<Bridge<L>>,
}

impl<L> PartialEq for StellationRootProps<L> {
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children && self.bridge == other.bridge
    }
}

impl<L> Clone for StellationRootProps<L>
where
    L: Link,
{
    fn clone(&self) -> Self {
        Self {
            children: self.children.clone(),
            bridge: self.bridge.clone(),
        }
    }
}

#[function_component]
pub(crate) fn StellationRoot<COMP, L>(props: &StellationRootProps<L>) -> Html
where
    COMP: BaseComponent,
    L: 'static + Link,
{
    let StellationRootProps { children, bridge } = props.clone();

    let get_init_states = use_callback(
        move |_, bridge| {
            let mut states = AnyMap::new();

            states.insert(BridgeState {
                inner: bridge.clone(),
            });

            states
        },
        bridge,
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
