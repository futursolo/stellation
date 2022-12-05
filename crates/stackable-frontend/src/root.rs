use bounce::helmet::HelmetBridge;
use bounce::BounceRoot;
use yew::prelude::*;
use yew_router::BrowserRouter;

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
            <BrowserRouter>
                <HelmetBridge />
                {children}
            </BrowserRouter>
        </BounceRoot>
    }
}
