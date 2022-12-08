use bounce::helmet::HelmetBridge;
use bounce::{use_atom_setter, BounceRoot};
use stackable_bridge::state::BridgeState;
use stackable_bridge::Bridge;
use yew::prelude::*;
use yew_router::BrowserRouter;

#[derive(Properties, PartialEq, Clone)]
pub struct StackableRootProps {
    #[prop_or_default]
    pub children: Html,
    pub bridge: Bridge,
}

#[function_component]
pub fn Inner(props: &StackableRootProps) -> Html {
    let StackableRootProps { children, bridge } = props.clone();
    let set_bridge = use_atom_setter::<BridgeState>();

    use_memo(
        move |_| {
            set_bridge(BridgeState { inner: bridge });
        },
        (),
    );

    html! {
        <BrowserRouter>
            <HelmetBridge />
            {children}
        </BrowserRouter>
    }
}

#[function_component]
pub fn StackableRoot<COMP>(props: &StackableRootProps) -> Html
where
    COMP: BaseComponent,
{
    let props = props.clone();

    html! {
        <BounceRoot>
            <Inner ..props />
        </BounceRoot>
    }
}
