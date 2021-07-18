# Project architecture

All non proc-macro related code is in `/packages/sycamore`. Proc-macro related code is in
`/packages/sycamore-macro`.

## Directory structure

- #### Reactivity

  - All the reactivity primitives are defined in `/packages/sycamore/src/reactive.rs`.

- #### `GenericNode`

  - `GenericNode` is a trait that serves as an abstraction for different rendering backends. Most
    commonly used types are `DomNode` for rendering in the browser to DOM nodes and `SsrNode` for
    rendering on the server to static HTML.

  - The `sycamore::utils::render` module contains backend agnostic utilities for rendering
    nodes.

- #### `Template`

  - `Template` is a wrapper type around a `GenericNode` that is produced by the `template!` macro. A
    `Template` can be rendered using the utilities in `sycamore::utils::render`.

- #### `template!`

  - The template macro is defined in `/packages/sycamore-macro/src/lib.rs`.

  - [`trybuild`](https://github.com/dtolnay/trybuild) is used for testing proc-macros.

## Fragment diffing

`Template` fragments are diffed in the `sycamore::utils::render::reconcile_fragments(_)`
method.

The diffing done by `Keyed` and `Indexed` is independent of the diffing done when rendering
fragments. Learn more about this in [`Iteration`](../basics/iteration).
