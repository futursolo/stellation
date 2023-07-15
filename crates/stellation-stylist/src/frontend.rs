use stylist::manager::StyleManager;
use stylist::yew::ManagerProvider;
use yew::html::ChildrenProps;
use yew::prelude::*;

/// A Stylist [`ManagerProvider`] that hydrates styles from SSR automatically.
/// This provider should be used in the client app instance.
///
/// # Panics
///
/// This provider requires a [`BackendManagerProvider`](crate::BackendManagerProvider) to be
/// placed in the server app or hydration will fail.
///
/// You can check out this [example](https://github.com/futursolo/stellation/blob/main/examples/fullstack/client/src/app.rs) for how to use this provider.
#[function_component]
pub fn FrontendManagerProvider(props: &ChildrenProps) -> Html {
    let manager = use_memo(
        |_| StyleManager::new().expect("failed to create style manager."),
        (),
    )
    .as_ref()
    .to_owned();

    html! {
        <ManagerProvider {manager}>
            {props.children.clone()}
        </ManagerProvider>
    }
}
