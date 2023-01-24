//! Core functionality for the Sycamore UI framework.
//!
//! This crate should not be used directly. Instead, use the `sycamore` crate which re-exports this
//! crate.
//!
//! # Feature flags
//!
//! - `hydrate`: Enables some machinery for hydrating the DOM. This is pretty specific to
//!   `sycamore-web` and is probably useless for any other backend.

#![deny(missing_debug_implementations)]

pub mod component;
pub mod elements;
pub mod generic_node;
pub mod noderef;
pub mod render;
pub mod view;
