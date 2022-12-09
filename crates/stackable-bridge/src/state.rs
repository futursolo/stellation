use bounce::Atom;

use crate::Bridge;

#[derive(Atom, PartialEq, Eq, Default, Debug)]
pub struct BridgeState {
    pub inner: Bridge,
}
