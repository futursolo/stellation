use bounce::Atom;

use crate::Bridge;

#[derive(Atom, PartialEq, Eq, Default)]
pub struct BridgeState {
    pub inner: Bridge,
}
