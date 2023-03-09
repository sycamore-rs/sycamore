//! Core functionality for the Sycamore UI framework.
//!
//! This crate should not be used directly. Instead, use the `sycamore` crate which re-exports this
//! crate.
//!
//! # Feature Flags
//!
//! - `hydrate` - Enables the hydration API.

#![deny(missing_debug_implementations)]

pub mod component;
pub mod event;
pub mod generic_node;
#[cfg(feature = "hydrate")]
pub mod hydrate;
pub mod noderef;
pub mod render;
pub mod stable_id;
pub mod view;
