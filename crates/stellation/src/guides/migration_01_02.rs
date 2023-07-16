//! Migration Guide from v0.1 to v0.2
//!
//! # 1. `stellation-backend` crate has been separated into multiple crates.
//!
//! `stellation-backend` has been separated into multiple crates:
//!
//! 1. `stellation-backend`: contains server renderer and other utilities to build backends.
//! 2. `stellation-backend-warp`: contains server that can be converted into a warp filter.
//! 3. `stellation-backend-tower`: contains server that can be converted into a tower service.
//! 4. `stellation-backend-cli`: contain out-of-the-box command line utility for backend
//!    applications.
//!
//! # 2. `stellation-bridge` has been rewritten.
//!
//! Previously, it uses feature flags to switch between local and remote bridges.
//! This has been switched to a link based system.
//!
//! Since this is a complete rewrite, we recommend users to refer to the new [fullstack](https://github.com/futursolo/stellation/tree/main/examples/fullstack) example about
//! how to use the new bridge.
//!
//! # 3. Bounce is updated to version v0.7
