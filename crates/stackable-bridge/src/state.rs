use std::rc::Rc;

use bounce::Atom;

use crate::{Bridge, BridgeMetadata};

#[derive(Atom, PartialEq, Eq, Default, Debug)]
pub struct BridgeState {
    pub inner: Bridge,
}

#[derive(Atom, Debug)]
pub struct BridgeMetadataState<CTX> {
    pub(crate) inner: Option<Rc<BridgeMetadata<CTX>>>,
}

impl<CTX> PartialEq for BridgeMetadataState<CTX> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<CTX> Default for BridgeMetadataState<CTX> {
    fn default() -> Self {
        Self { inner: None }
    }
}

impl<CTX> From<Rc<BridgeMetadata<CTX>>> for BridgeMetadataState<CTX> {
    fn from(m: Rc<BridgeMetadata<CTX>>) -> Self {
        Self { inner: Some(m) }
    }
}
