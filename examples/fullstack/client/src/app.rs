use example_fullstack_view::Main;
use stylist::manager::StyleManager;
use stylist::yew::ManagerProvider;
use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    let manager = use_memo(
        |_| StyleManager::new().expect("failed to create style manager."),
        (),
    )
    .as_ref()
    .to_owned();

    html! {
        <Suspense fallback={Html::default()}>
            <ManagerProvider {manager}>
                <Main />
            </ManagerProvider>
        </Suspense>
    }
}
