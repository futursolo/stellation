#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

use yew::prelude::*;

mod pages;
use pages::{Greeting, ServerTime};

#[function_component]
pub fn Main() -> Html {
    let fallback = html! {<div class="time-loading">{"Loading..."}</div>};

    html! {
        <div class="container">
            <div class="title">{"Welcome to Stackable!"}</div>
            <Suspense {fallback}>
                <ServerTime />
            </Suspense>
            <Greeting />
        </div>
    }
}
