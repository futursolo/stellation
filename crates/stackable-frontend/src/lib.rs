use std::marker::PhantomData;

use stackable_bridge::Bridge;
use yew::prelude::*;

use crate::root::{StackableRoot, StackableRootProps};
mod root;

pub struct Renderer<COMP>
where
    COMP: BaseComponent,
{
    props: COMP::Properties,
    bridge: Option<Bridge>,
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
    pub fn new() -> Renderer<COMP>
    where
        COMP::Properties: Default,
    {
        Self::with_props(Default::default())
    }

    pub fn with_props(props: COMP::Properties) -> Renderer<COMP> {
        Renderer {
            props,
            bridge: None,
            _marker: PhantomData,
        }
    }

    pub fn bridge(mut self, bridge: Bridge) -> Self {
        self.bridge = Some(bridge);

        self
    }

    fn into_yew_renderer(self) -> yew::Renderer<StackableRoot<COMP>> {
        let Self { props, bridge, .. } = self;
        let bridge = bridge.unwrap_or_default();

        let children = html! {
            <COMP ..props />
        };

        let props = StackableRootProps { bridge, children };

        yew::Renderer::with_props(props)
    }

    pub fn render(self) {
        self.into_yew_renderer().render();
    }

    pub fn hydrate(self) {
        self.into_yew_renderer().hydrate();
    }
}
