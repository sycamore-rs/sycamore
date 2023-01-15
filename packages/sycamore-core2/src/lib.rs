//! Core functionality for the Sycamore UI framework.
//!
//! This crate should not be used directly. Instead, use the `sycamore` crate which re-exports this
//! crate.
//!
//! # Feature Flags
//!
//! - `hydrate` - Enables the hydration API.

#![deny(missing_debug_implementations)]

pub mod attributes;
pub mod component;
pub mod event;
pub mod generic_node;
pub mod noderef;
pub mod render;
pub mod view;
