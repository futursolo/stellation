use std::fmt;
use std::fmt::Write;
use std::marker::PhantomData;
use std::rc::Rc;

use bounce::helmet::render_static;
use stellation_bridge::links::{Link, PhantomLink};
use stellation_bridge::Bridge;
use yew::BaseComponent;

use crate::hooks::HeadContents;
use crate::request::RenderRequest;
use crate::root::{StellationRoot, StellationRootProps};
use crate::{html, ServerAppProps};

/// The Stellation Backend Renderer.
///
/// This type wraps the [Yew Server Renderer](yew::ServerRenderer) and provides additional features.
///
/// # Note
///
/// Stellation provides [`BrowserRouter`](yew_router::BrowserRouter) and
/// [`BounceRoot`](bounce::BounceRoot) to all applications.
///
/// Bounce Helmet is also bridged automatically.
///
/// You do not need to add them manually.
pub struct ServerRenderer<COMP, REQ = (), CTX = (), L = PhantomLink>
where
    COMP: BaseComponent,
{
    request: REQ,
    bridge: Option<Bridge<L>>,
    _marker: PhantomData<(COMP, REQ, CTX)>,
}

impl<COMP, REQ, CTX, L> fmt::Debug for ServerRenderer<COMP, REQ, CTX, L>
where
    COMP: BaseComponent,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ServerRenderer<_>")
    }
}

impl<COMP, REQ, CTX> ServerRenderer<COMP, REQ, CTX>
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX, REQ>>,
{
    /// Creates a Renderer with specified request.
    pub fn new(request: REQ) -> ServerRenderer<COMP, REQ, CTX> {
        ServerRenderer {
            request,
            bridge: None,
            _marker: PhantomData,
        }
    }
}

impl<COMP, REQ, CTX, L> ServerRenderer<COMP, REQ, CTX, L>
where
    COMP: BaseComponent<Properties = ServerAppProps<CTX, REQ>>,
{
    /// Connects a bridge to the application.
    pub fn bridge<T>(self, bridge: Bridge<T>) -> ServerRenderer<COMP, REQ, CTX, T> {
        ServerRenderer {
            request: self.request,
            bridge: Some(bridge),
            _marker: PhantomData,
        }
    }

    /// Renders the application.
    ///
    /// # Note:
    ///
    /// This future is `!Send`.
    pub async fn render(self) -> String
    where
        CTX: 'static,
        REQ: 'static,
        L: 'static + Link,
        REQ: RenderRequest<Context = CTX>,
    {
        let Self {
            bridge, request, ..
        } = self;

        let mut head_s = String::new();
        let mut helmet_tags = Vec::new();
        let mut body_s = String::new();

        let request: Rc<_> = request.into();

        if !request.is_client_only() {
            let head_contents = HeadContents::new();

            let (reader, writer) = render_static();

            let props = ServerAppProps::from_request(request.clone());

            body_s = yew::LocalServerRenderer::<StellationRoot<COMP, CTX, REQ, L>>::with_props(
                StellationRootProps {
                    server_app_props: props,
                    helmet_writer: writer,
                    bridge,
                    head_contents: head_contents.clone(),
                },
            )
            .render()
            .await;

            helmet_tags.append(&mut reader.render().await);
            let _ = write!(
                &mut head_s,
                r#"<meta name="stellation-mode" content="hydrate">"#
            );

            head_contents.render_into(&mut head_s).await;
        }

        html::format_html(request.template(), helmet_tags, head_s, body_s).await
    }
}
