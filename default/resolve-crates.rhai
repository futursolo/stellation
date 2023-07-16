
//! Resolves stellation crates
//!
//! This template has three targets:
//!
//! - `main`: the template targets the main branch.
//! - `release`: the template targets the release set in stellation_release_ver.
//! - `custom`: the template will exclude stellation crates, they should be added manually.
//! - `ci`: the template is running under GitHub Actions to verify the template.

// Sets the default target to "main"
variable::set("stellation_target", "main");
variable::set("stellation_release_ver", "");

// Additional definition
variable::set("stellation_target", "release");
variable::set("stellation_release_ver", "0.2.0");
