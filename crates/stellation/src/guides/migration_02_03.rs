//! Migration Guide from v0.2 to v0.3
//!
//! # 1. Bounce is updated to version v0.8
//!
//! # 2. bridged mutations and queries now exposes their state.
//!
//! Previously, mutations and queries exposes 2 methods on their handle `result()` and `status()`.
//! The status returns an enum where it can only be used to check the status of the query /
//! mutation. In Bounce v0.8, it has been modified to return a `State` Enum where the completed and
//! outdated variant contains the query / mutation result. The stellation bridge has been updated to
//! follow this new convention.
