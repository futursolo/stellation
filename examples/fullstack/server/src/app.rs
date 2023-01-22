use example_fullstack_view::Main;
use stellation_backend::{Request, ServerAppProps};
use yew::prelude::*;

#[function_component]
pub fn ServerApp<REQ>(_props: &ServerAppProps<(), REQ>) -> Html
where
    REQ: Request,
{
    html! {
        <Main />
    }
}
