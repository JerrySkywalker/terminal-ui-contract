//! Read-only scheduling and snapshot acceptance. Product truth remains outside this crate.
#![forbid(unsafe_code)]
pub mod lifecycle;
pub mod refresh;
pub mod snapshot;
