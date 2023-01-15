#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

use bounce::helmet::Helmet;
use yew::prelude::*;

mod pages;
use pages::{Greeting, ServerTime};

#[function_component]
pub fn Main() -> Html {
    let fallback = html! {<div class="time-loading">{"Loading..."}</div>};

    html! {
        <>
            <Helmet>
                <title>{"Welcome to Stellation!"}</title>
            </Helmet>
            <div class="container">
                <div class="title">{"Welcome to Stellation!"}</div>
                <Suspense {fallback}>
                    <ServerTime />
                </Suspense>
                <Greeting />
            </div>
        </>
    }
}
