use stellation_backend::ServerAppProps;
use yew::prelude::*;

use crate::view::Main;

#[function_component]
pub fn ServerApp(_props: &ServerAppProps<()>) -> Html {
    html! {
        <Main />
    }
}
