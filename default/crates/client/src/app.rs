use stellation_stylist::FrontendManagerProvider;
use yew::prelude::*;

use crate::view::Main;

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
