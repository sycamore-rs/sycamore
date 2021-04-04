# Project architecture

All non proc-macro related code is in `/maple-core`.
Proc-macro related code is in `/maple-core-macro`.

## Concepts and where to find them

- #### Reactivity

  - All the reactivity primitives are defined in `/maple-core/src/reactive.rs`.
  - `create_effect` is called by the internal DOM utilities in `/maple-core/src/internal.rs`.

- #### `template!`

  - The template macro is defined in `/maple-core-macro/src/lib.rs`.
  - Different DOM nodes are defined in separate files under the same directory.
  - [`trybuild`](https://github.com/dtolnay/trybuild) is used for testing proc-macros.

- #### Components
  - Components are just functions! There is no special code for handling components at runtime.
