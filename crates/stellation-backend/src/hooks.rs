//! Useful hooks for stellation backend.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use bounce::{use_atom_value, Atom};
use futures::future::LocalBoxFuture;
use futures::{Future, FutureExt};
use yew::prelude::*;

type RenderAppendHead = Box<dyn FnOnce() -> LocalBoxFuture<'static, String>>;

#[derive(Atom, Clone)]
pub(crate) struct HeadContents {
    inner: Rc<RefCell<Vec<RenderAppendHead>>>,
}

impl Default for HeadContents {
    fn default() -> Self {
        panic!("Attempting to use use_append_head_content on client side rendering!");
    }
}

impl PartialEq for HeadContents {
    // We never set this atom, so it will always be equal.
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for HeadContents {}

impl HeadContents {
    pub(crate) fn new() -> Self {
        Self {
            inner: Rc::default(),
        }
    }

    pub(crate) async fn render_into(&self, w: &mut dyn fmt::Write) {
        for i in self.inner.take() {
            let _ = write!(w, "{}", i().await);
        }
    }
}

/// A server-side hook that appends content to head element.
/// This async function is resolved after the page is completed rendered and the returned string is
/// appended at the location of the ` <!--%STELLATION_HEAD%-->` comment in `index.html`, after other
/// contents.
///
/// # Warning
///
/// The content is not managed at the client side.
/// This hook is used to facility specific crates such as a CSS-in-Rust solution.
///
/// If you wish to render content into the `<head>` element, you should use
/// [`bounce::helmet::Helmet`].
///
///
/// # Panics
///
/// This hook should be used by a server-side only component. Panics if used in client-side
/// rendering.
#[hook]
pub fn use_append_head_content<F, Fut>(f: F)
where
    F: 'static + FnOnce() -> Fut,
    Fut: 'static + Future<Output = String>,
{
    let boxed_f: RenderAppendHead = Box::new(move || f().boxed_local());
    let head_contents = use_atom_value::<HeadContents>();

    head_contents.inner.borrow_mut().push(boxed_f);
}
