
//! Resolves stackable crates
//!
//! This template has three targets:
//!
//! - `main`: the template targets the main branch.
//! - `release`: the template targets the release set in stackable_release_ver.
//! - `custom`: the template will exclude stackable crates, they should be added manually.

// Sets the default target to "main"
variable::set("stackable_target", "main");
variable::set("stackable_release_ver", "");

// Additional definition
variable::set("stackable_target", "main");
