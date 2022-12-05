use bounce::helmet::HelmetBridge;
use bounce::BounceRoot;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct StackableRootProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn StackableRoot(props: &StackableRootProps) -> Html {
    let StackableRootProps { children } = props.clone();
    html! {
        <BounceRoot>
            <HelmetBridge />
            {children}
        </BounceRoot>
    }
}
