use std::ops::Deref;

use yew::html::ChildrenProps;
use yew::prelude::*;

/// A component that automatically excludes its children from server-side rendering.
#[function_component]
pub fn ClientOnly(props: &ChildrenProps) -> Html {
    let should_render = use_state(|| false);

    // Effects are only run on the client side.
    {
        use_effect_with_deps(
            |should_render_setter| {
                should_render_setter.set(true);
            },
            should_render.setter(),
        );
    }

    match should_render.deref() {
        true => html! {<>{props.children.clone()}</>},
        false => Html::default(),
    }
}
