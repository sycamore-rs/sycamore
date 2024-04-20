//! Core functionality for the Sycamore UI framework.
//!
//! This crate should not be used directly. Instead, use the `sycamore` crate which re-exports this
//! crate.

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

mod component;

pub use component::*;
