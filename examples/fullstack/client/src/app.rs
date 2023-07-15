use example_fullstack_view::Main;
use stellation_stylist::FrontendManagerProvider;
use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    html! {
        <Suspense fallback={Html::default()}>
            <FrontendManagerProvider>
                <Main />
            </FrontendManagerProvider>
        </Suspense>
    }
}
