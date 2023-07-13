#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

use bounce::helmet::Helmet;
use stylist::yew::{styled_component, Global};
use yew::prelude::*;

mod pages;
use pages::{Greeting, ServerTime};

#[styled_component]
pub fn Main() -> Html {
    let fallback = html! {<div class="time-loading">{"Loading..."}</div>};

    html! {
        <>
            <Global css={css!(r#"
                html,
                body {
                    margin: 0;
                    padding: 0;

                    font-family: Verdana, Geneva, Tahoma, sans-serif;
                    font-size: 15px;
                }

                @media (prefers-color-scheme: dark) {
                    html {
                        background-color: rgb(50, 50, 50);
                        color: white;
                    }
                }
            "#)} />
            <Helmet>
                <title>{"Welcome to Stellation!"}</title>
            </Helmet>
            <div class={css!(r#"
                height: 100vh;
                width: 100%;

                display: flex;
                flex-direction: column;
                justify-content: center;
                align-items: center;
            "#)}>
                <div class={css!(r#"
                    font-size: 2rem;
                    line-height: 1.5em;
                "#)}>
                    {"Welcome to Stellation!"}
                </div>
                <Suspense {fallback}>
                    <ServerTime />
                </Suspense>
                <Greeting />
            </div>
        </>
    }
}
