---
title: Announcing Sycamore v0.6.0
description: Faster and faster with plenty of fixes and features...
date: 2021-09-12
---

# Announcing Sycamore v0.6.0!

_Faster and faster with plenty of fixes and features..._

Hello everybody! Sycamore is a library for building isomorphic web applications
in Rust and WebAssembly.

I'm happy to announce that we've just released v0.6.0. This release contains
plenty of fixes, shiny new features, and QOL improvements.

(BTW, we just reached 300+ stars on GitHub! Thanks everybody! 🎉)

## What's New?

### Perseus

[Perseus](https://github.com/arctic-hen7/perseus) (by community member
[`@arctic-hen7`](https://github.com/arctic-hen7)) is a new web framework for
building websites with Rust. Think [NextJS](https://nextjs.org/) or
[SvelteKit](https://kit.svelte.dev/) but with no JavaScript.

Perseus supports, among other things:

- static generation (serving only static resources)
- server-side rendering (serving dynamic resources)
- revalidation after time and/or with custom logic (updating rendered pages)
- incremental regeneration (build on demand)
- open build matrix (use any rendering strategy with anything else, mostly)
- CLI harness that lets you build apps with ease and confidence
- full i18n support out-of-the-box with [Fluent](https://projectfluent.org/)

And also, it's built on top of Sycamore. Go check it out!

### Higher-order components

Components can now be generic over other components! This allows interesting
patterns, similar to higher-order components in React.

```rust
#[component(EnhancedComponent<G>)]
fn enhanced_component<C: Component<G, Props = i32>>() -> Template<G> {
    template! {
        div(class="enhanced-container") {
            C(42)
        }
    }
}
```

It will be interesting to see how this feature can be used in practice.

### Boolean attributes

Previously, using boolean attributes could be quiet annoying. Oftentimes, you
needed two separate code branches with almost identical code.

```rust
// Before
template! {
    (if *my_signal.get() {
        template! {
            button(disabled="") { ... }
        }
    } else {
        template! {
            button() { ... }
        }
    })
}
// After
template! {
    button(disabled=*my_signal.get()) { ... }
}
```

Ah... so much nicer!

### Separate `sycamore-reactive` crate

The reactive primitives that power `sycamore` have been extracted into a new
crate: [`sycamore-reactive`](https://crates.io/crates/sycamore-reactive). This
allows using these powerful sycamore primitives outside of the main `sycamore`
crate.

### Performance improvements

A lot of performance improvements have been made to Sycamore.

<img src="https://user-images.githubusercontent.com/37006668/143656692-df777e44-a7fa-4cb2-ae8b-62e15c75968e.png" alt="js-framework-benchmark screenshot" style="max-width: 500px" />

Sycamore is now faster than most major JavaScript frameworks!

## Conclusion

A big thank you to all the
[contributors](https://github.com/sycamore-rs/sycamore/graphs/contributors) who
made this release possible!

For more detailed changes, check out the
[changelog](https://github.com/sycamore-rs/sycamore/blob/master/CHANGELOG.md#-060-2021-09-12).

If you are interested in contributing to Sycamore, check out the issues labeled
with
[`good first issue`](https://github.com/sycamore-rs/sycamore/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)
on our GitHub repository. Don't hesitate to join our
[Discord server](https://discord.gg/vDwFUmm6mU) too! See you there.

Thanks!
