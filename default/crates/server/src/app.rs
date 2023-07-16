use stellation_backend::{Request, ServerAppProps};
use stellation_stylist::BackendManagerProvider;
use yew::prelude::*;

use crate::view::Main;

#[function_component]
pub fn ServerApp<REQ>(_props: &ServerAppProps<(), REQ>) -> Html
where
    REQ: Request,
{
    html! {
        <Suspense fallback={Html::default()}>
            <BackendManagerProvider>
                <Main />
            </BackendManagerProvider>
        </Suspense>
    }
}
