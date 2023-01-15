use stellation_bridge::hooks::use_bridged_mutation;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::api::GreetingMutation;

#[function_component]
pub fn Greeting() -> Html {
    let handle = use_bridged_mutation::<GreetingMutation>();

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
        <div class="greeting-container">
            <div class="greeting-info">
                <input
                    class="greeting-input"
                    type="text"
                    placeholder="Your Name"
                    required={true}
                    value={name.to_string()}
                    minlength="1"
                    {oninput}
                    ref={input_ref}
                />
                <button class="greeting-button" {onclick}>{"Hello Stellation!"}</button>
            </div>
            <div class="greeting-message">{message}</div>
        </div>
    }
}
