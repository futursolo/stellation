use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bounce::{BounceStates, Selector};

use crate::links::Link;

pub(super) type ReadToken = Box<dyn Fn(&BounceStates) -> Rc<dyn AsRef<str>>>;

/// The Bridge.
pub struct Bridge<L> {
    id: usize,
    pub(crate) link: L,
    read_token: Option<ReadToken>,
}

impl<L> fmt::Debug for Bridge<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bridge")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl<L> PartialEq for Bridge<L> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<L> Eq for Bridge<L> {}

impl<L> Bridge<L>
where
    L: Link,
{
    /// Creates a new Bridge.
    pub fn new(link: L) -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);
        let id = ID.fetch_add(1, Ordering::AcqRel);

        Self {
            id,
            link,
            read_token: None,
        }
    }

    pub(crate) fn read_token(&self, states: &BounceStates) -> Option<Rc<dyn AsRef<str>>> {
        self.read_token.as_ref().map(|m| m(states))
    }

    /// Selects the token from a bounce state.
    pub fn with_token_selector<T>(mut self) -> Self
    where
        T: 'static + Selector + AsRef<str>,
    {
        let read_token = Box::new(move |states: &BounceStates| {
            let state = states.get_selector_value::<T>();

            state as Rc<dyn AsRef<str>>
        }) as ReadToken;

        self.read_token = Some(read_token);

        self
    }
}
