#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

use yew::prelude::*;

mod pages;
use pages::ServerTime;

#[function_component]
pub fn Main() -> Html {
    let fallback = html! {<div class="time-loading">{"Loading..."}</div>};

    html! {
        <div class="time-container">
            <div class="time-title">{"Stackable!"}</div>
            <Suspense {fallback}>
                <ServerTime />
            </Suspense>
        </div>
    }
}
