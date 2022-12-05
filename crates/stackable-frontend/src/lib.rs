use std::marker::PhantomData;

use yew::prelude::*;

use crate::root::{StackableRoot, StackableRootProps};
mod root;

pub struct Renderer<COMP>
where
    COMP: BaseComponent,
{
    inner: yew::Renderer<StackableRoot>,
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
        let children = html! {
            <COMP ..props />
        };

        Renderer {
            inner: yew::Renderer::with_props(StackableRootProps { children }),
            _marker: PhantomData,
        }
    }

    pub fn render(self) {
        self.inner.render();
    }

    pub fn hydrate(self) {
        self.inner.hydrate();
    }
}
