//! Dependency-neutral terminal Human interface primitives.
//!
//! This crate intentionally exposes no Ratatui or Crossterm type. Consumers
//! adapt its semantic state and tokens to their selected terminal stack.

#![forbid(unsafe_code)]

