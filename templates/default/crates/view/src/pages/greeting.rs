use stylist::yew::styled_component;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::api::{Bridge, GreetingMutation};

#[styled_component]
pub fn Greeting() -> Html {
    let handle = Bridge::use_mutation::<GreetingMutation>();

    let message = match handle.result() {
        None => "".to_string(),
        Some(Ok(m)) => m.message.to_string(),
        Some(Err(_)) => "failed to communicate with server...".into(),
    };

    let input_ref = use_node_ref();

    let name = use_state_eq(|| "".to_string());
    let oninput = use_callback(
        |input: InputEvent, set_value| {
            let el = input.target_unchecked_into::<HtmlInputElement>();
            set_value.set(el.value());
        },
        name.setter(),
    );

    let onclick = {
        let input_ref = input_ref.clone();

        use_callback(
            move |_input, name| {
                if !input_ref
                    .cast::<HtmlInputElement>()
                    .map(|m| m.report_validity())
                    .unwrap_or(false)
                {
                    return;
                }

                let name = name.clone();
                let handle = handle.clone();
                spawn_local(async move {
                    let _ = handle.run(name.to_string()).await;
                });
            },
            name.clone(),
        )
    };

    html! {
        <div class={css!(r#"
            padding-top: 2rem;
            max-width: 300px;
            width: calc(100% - 20px);
        "#)}>
            <div class="greeting-info">
                <input
                    class={css!(r#"
                        width: 100%;
                        height: 40px;

                        display: block;
                        box-sizing: border-box;

                        border-radius: 8px;

                        background-color: rgb(230, 226, 245);
                        color: rgb(0, 0, 0);

                        border: 0;
                        outline: 0;
                        padding-left: 1rem;
                        padding-right: 1rem;

                        font-size: 1rem;

                        &::-webkit-input-placeholder,
                        &::-moz-placeholder,
                        &:-moz-placeholder,
                        &:-ms-input-placeholder {
                            color: rgb(206, 206, 206);
                        }

                        @media (prefers-color-scheme: dark) {
                            background-color: rgb(87, 86, 91);
                            color: white;

                            &::-webkit-input-placeholder,
                            &::-moz-placeholder,
                            &:-moz-placeholder,
                            &:-ms-input-placeholder {
                                color: rgb(181, 181, 181);
                            }
                        }

                    "#)}
                    type="text"
                    placeholder="Your Name"
                    required={true}
                    value={name.to_string()}
                    minlength="1"
                    {oninput}
                    ref={input_ref}
                />
                <button
                    class={css!(r#"
                        width: 100%;
                        height: 40px;

                        margin: 0;
                        padding: 0;
                        margin-top: 1rem;

                        display: block;

                        border-radius: 8px;

                        background-color: rgb(132, 112, 198);
                        color: white;
                        border: 0;

                        cursor: pointer;

                        font-size: 1rem;
                        font-weight: bold;

                        @media (prefers-color-scheme: dark) {
                            background-color: rgb(95, 76, 159);
                        }
                    "#)}
                    {onclick}
                >{"Hello Stellation!"}</button>
            </div>
            <div class={css!(r#"
                padding-top: 1rem;
                height: 2rem;
                box-sizing: border-box;
                text-align: center;
            "#)}>{message}</div>
        </div>
    }
}
