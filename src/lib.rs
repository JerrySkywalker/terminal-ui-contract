//! Dependency-neutral terminal Human interface primitives.
//!
//! This crate intentionally exposes no Ratatui or Crossterm type. Consumers
//! adapt its semantic state and tokens to their selected terminal stack.

#![forbid(unsafe_code)]

pub mod chrome;
pub mod footer;
pub mod input;
pub mod interaction;
pub mod layout;
pub mod locale;
pub mod navigation;
pub mod text;
