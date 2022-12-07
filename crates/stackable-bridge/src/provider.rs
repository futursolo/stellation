use yew::prelude::*;

use crate::Bridge;

#[derive(Properties, PartialEq, Clone)]
pub struct BridgeProviderProps {
    #[prop_or_default]
    pub children: Children,
    pub bridge: Bridge,
}

#[function_component]
pub fn BridgeProvider(props: &BridgeProviderProps) -> Html {
    let BridgeProviderProps { children, bridge } = props.clone();

    html! {
        <ContextProvider<Bridge> context={bridge}>
            {children}
        </ContextProvider<Bridge>>
    }
}
