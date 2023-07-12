//! Stellation Frontend.
//!
//! This crate contains the frontend renderer and useful utilities for stellation applications.

#![deny(clippy::all)]
#![deny(missing_debug_implementations)]
#![deny(unsafe_code)]
#![deny(non_snake_case)]
#![deny(clippy::cognitive_complexity)]
#![deny(missing_docs)]
#![cfg_attr(documenting, feature(doc_cfg))]
#![cfg_attr(documenting, feature(doc_auto_cfg))]
#![cfg_attr(any(releasing, not(debug_assertions)), deny(dead_code, unused_imports))]

use std::marker::PhantomData;

use bounce::Selector;
use stellation_bridge::links::{Link, PhantomLink};
use stellation_bridge::state::BridgeState;
use stellation_bridge::Bridge;
use yew::prelude::*;

use crate::root::{StellationRoot, StellationRootProps};
pub mod components;
mod root;
pub mod trace;

/// The Stellation Frontend Renderer.
///
/// This type wraps the [Yew Renderer](yew::Renderer) and provides additional features.
///
/// # Note
///
/// Stellation provides [`BrowserRouter`](yew_router::BrowserRouter) and
/// [`BounceRoot`](bounce::BounceRoot) to all applications.
///
/// Bounce Helmet is also bridged automatically.
///
/// You do not need to add them manually.
#[derive(Debug)]
pub struct Renderer<COMP, L = PhantomLink>
where
    COMP: BaseComponent,
    L: Link,
{
    props: COMP::Properties,
    bridge_state: Option<BridgeState<L>>,
    _marker: PhantomData<COMP>,
}

impl<COMP> Default for Renderer<COMP>
where
    COMP: BaseComponent,
    COMP::Properties: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<COMP> Renderer<COMP>
where
    COMP: BaseComponent,
{
    /// Creates a Renderer with default props.
    pub fn new() -> Renderer<COMP>
    where
        COMP::Properties: Default,
    {
        Self::with_props(Default::default())
    }
}

impl<COMP, L> Renderer<COMP, L>
where
    COMP: BaseComponent,
    L: 'static + Link,
{
    /// Creates a Renderer with specified props.
    pub fn with_props(props: COMP::Properties) -> Renderer<COMP, L> {
        Renderer {
            props,
            bridge_state: None,
            _marker: PhantomData,
        }
    }

    /// Connects a bridge to the application.
    pub fn bridge_selector<S, LINK>(self) -> Renderer<COMP, LINK>
    where
        S: 'static + Selector + AsRef<Bridge<LINK>>,
        LINK: 'static + Link,
    {
        Renderer {
            props: self.props,
            bridge_state: Some(BridgeState::from_bridge_selector::<S>()),
            _marker: PhantomData,
        }
    }

    fn into_yew_renderer(self) -> yew::Renderer<StellationRoot<L>> {
        let Self {
            props,
            bridge_state,
            ..
        } = self;

        let children = html! {
            <COMP ..props />
        };

        let props = StellationRootProps {
            bridge_state,
            children,
        };

        yew::Renderer::with_props(props)
    }

    /// Renders the application.
    ///
    /// Whether the application is rendered or hydrated is determined automatically based on whether
    /// SSR is used on the server side for this page.
    pub fn render(self) {
        let renderer = self.into_yew_renderer();

        if web_sys::window()
            .and_then(|m| m.document())
            .and_then(|m| {
                m.query_selector(r#"meta[name="stellation-mode"][content="hydrate"]"#)
                    .ok()
                    .flatten()
            })
            .is_some()
        {
            renderer.hydrate();
        } else {
            renderer.render();
        }
    }
}
