use example_fullstack_view::Main;
use stellation_backend::ServerAppProps;
use yew::prelude::*;

#[function_component]
pub fn ServerApp(_props: &ServerAppProps<()>) -> Html {
    html! {
        <Main />
    }
}
