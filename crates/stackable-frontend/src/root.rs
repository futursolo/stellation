use bounce::helmet::HelmetBridge;
use bounce::BounceRoot;
use stackable_bridge::provider::BridgeProvider;
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
pub fn StackableRoot<COMP>(props: &StackableRootProps) -> Html
where
    COMP: BaseComponent,
{
    let StackableRootProps { children, bridge } = props.clone();

    html! {
        <BounceRoot>
            <BridgeProvider {bridge}>
                <BrowserRouter>
                    <HelmetBridge />
                    {children}
                </BrowserRouter>
            </BridgeProvider>
        </BounceRoot>
    }
}
