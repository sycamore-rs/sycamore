---
title: Roadmap
---

# Roadmap

This roadmap provides some ideas of the general direction we're heading with
Sycamore and what to expect. Any item here is not a promise and is subject to
change or removal, depending on circumstances.

## Future

### v0.9.x minor release

- [ ] Better server integrations (this might belong in a different repo)
  - [ ] Server functions
  - [ ] Serializing and deserializing isomorphic resources
  - [ ] First-party integrations for popular Rust servers such as Actix, Axum,
        Rocket, etc.
- [ ] Iron out some rough edges in `sycamore-router`
  - [ ] Router reloading
        ([#335](https://github.com/sycamore-rs/sycamore/issues/335))
  - [ ] Scroll position saving
        ([#336](https://github.com/sycamore-rs/sycamore/issues/336))

### v0.10

- [ ] Sycamore CLI
  - [ ] Investigate Hot Module Reloading (HMR), at least for static views
  - [ ] Integration with CSS, SCSS, Tailwind, and perhaps other assets. Perhaps
        we can use [manganis](https://github.com/DioxusLabs/manganis) form
        Dioxus?
  - [ ] Integration with `wasm-split`. Figure out how to combine code-splitting
        with Router.
- [ ] Islands Architecture
      ([#200](https://github.com/sycamore-rs/sycamore/issues/200))
  - [ ] Server components
- [ ] Synthetic event delegation
      ([#176](https://github.com/sycamore-rs/sycamore/issues/176))
- [ ] Fallible components
      ([#528](https://github.com/sycamore-rs/sycamore/issues/528))
- [ ] Investigate other possible rendering backends (TUI, Native, Liveview?)

## Completed
