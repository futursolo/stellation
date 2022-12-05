use bounce::helmet::{HelmetBridge, StaticWriter};
use bounce::BounceRoot;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct StackableRootProps {
    #[prop_or_default]
    pub children: Html,
    pub helmet_writer: StaticWriter,
}

#[function_component]
pub fn StackableRoot(props: &StackableRootProps) -> Html {
    let StackableRootProps {
        children,
        helmet_writer,
    } = props.clone();
    html! {
        <BounceRoot>
            <HelmetBridge writer={helmet_writer} />
            {children}
        </BounceRoot>
    }
}
