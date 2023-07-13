use std::cell::RefCell;
use std::rc::Rc;

use example_fullstack_view::Main;
use stellation_backend::hooks::use_append_head_content;
use stellation_backend::{Request, ServerAppProps};
use stylist::manager::{render_static, StyleManager};
use stylist::yew::ManagerProvider;
use yew::prelude::*;

#[function_component]
pub fn ServerApp<REQ>(_props: &ServerAppProps<(), REQ>) -> Html
where
    REQ: Request,
{
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
        <Suspense fallback={Html::default()}>
            <ManagerProvider {manager}>
                <Main />
            </ManagerProvider>
        </Suspense>
    }
}
