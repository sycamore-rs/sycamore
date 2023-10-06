# Architecture

TODO: this is missing a lot of things

## Directory structure

* Main crates (including `sycamore`, `sycamore-reactive`, etc...) are in `packages/`.
* Integration tests are in `packages/sycamore/tests`.
* Benchmarks are in `packages/tools/bench`.
* Examples are in `examples/`.
* The Sycamore website is in `website/`. This will eventually be moved out into a new repository.
* The documentation is in `docs/`. This also contains a tool for pre-rendering the markdown files into HTMl.

## Crates

### `sycamore`

This is the main crate which is intended to be added to the `Cargo.toml` of a Sycamore project. This crate re-exports most of the APIs of the other crates.

### `sycamore-macro`

This crate contains all the proc-macro logic for `view!`, `#[component]`, and `#[derive(Props)]`.

### `sycamore-reactive`

This is the backbone of Sycamore's reactivity system. This crate can be used stand-alone without any of the other crates.

### `sycamore-core`

This crate contains all the core utilities for Sycamore's rendering logic. This includes, for example, machinery used by the component system and the view fragment diffing logic.
This crate is backend-agnostic, meaning that there should be no dependence on `web-sys` or `wasm-bindgen`.

### `sycamore-web`

This crate contains all the web specific rendering logic for Sycamore.

### `sycamore-futures`

A lightweight crate to choose between `tokio` when on the server and `wasm-bindgen-futures` when on the client.

### `sycamore-router` and `sycamore-router-macro`

This is an implementation of a SPA router for Sycamore. This will eventually be moved into a new repository.

## How reactivity works

TOOD: explain reactivity system behind the scenes, including update propagation and reactive scopes.
