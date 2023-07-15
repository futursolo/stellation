use std::cell::RefCell;
use std::rc::Rc;

use stellation_backend::hooks::use_append_head_content;
use stylist::manager::{render_static, StyleManager};
use stylist::yew::ManagerProvider;
use yew::html::ChildrenProps;
use yew::prelude::*;

/// A Stylist [`ManagerProvider`] that writes server-side styles to
/// [`ServerRenderer`](stellation_backend::ServerRenderer) automatically.
///
/// # Panics
///
/// This provider should be used in the server app instance. Using this component in the client app
/// will panic.
///
/// This provider requires a [`FrontendManagerProvider`](crate::FrontendManagerProvider) to be
/// placed in the client app or hydration will fail.
///
/// You can check out this [example](https://github.com/futursolo/stellation/blob/main/examples/fullstack/server/src/app.rs) for how to use this provider.
#[function_component]
pub fn BackendManagerProvider(props: &ChildrenProps) -> Html {
    let (reader, manager) = use_memo(
        |_| {
            let (writer, reader) = render_static();

            let style_mgr = StyleManager::builder()
                .writer(writer)
                .build()
                .expect("failed to create style manager.");

            (Rc::new(RefCell::new(Some(reader))), style_mgr)
        },
        (),
    )
    .as_ref()
    .to_owned();

    use_append_head_content(move || async move {
        let style_data = reader
            .borrow_mut()
            .take()
            .expect("reader is called twice!")
            .read_style_data();

        let mut s = String::new();
        // Write to a String can never fail.
        let _ = style_data.write_static_markup(&mut s);

        s
    });

    html! {
        <ManagerProvider {manager}>
            {props.children.clone()}
        </ManagerProvider>
    }
}
