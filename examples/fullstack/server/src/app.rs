use example_fullstack_app::Content;
use stackable_backend::ServerAppProps;
use yew::prelude::*;

#[function_component]
pub fn ServerApp(_props: &ServerAppProps<()>) -> Html {
    html! {
        <Content />
    }
}
