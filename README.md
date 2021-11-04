# Sycamore

[![Crates.io](https://img.shields.io/crates/v/sycamore)](https://crates.io/crates/sycamore)
[![docs.rs](https://img.shields.io/docsrs/sycamore?color=blue&label=docs.rs)](https://docs.rs/sycamore)
[![GitHub contributors](https://img.shields.io/github/contributors/sycamore-rs/sycamore)](https://github.com/sycamore-rs/sycamore/graphs/contributors)
[![Discord](https://img.shields.io/discord/820400041332179004?label=discord)](https://discord.gg/vDwFUmm6mU)

## What is Sycamore?

Sycamore is a modern VDOM-less web library with fine-grained reactivity.

- **Lightning Speed**: Sycamore harnesses the full power of [Rust](https://www.rust-lang.org/) via
  [WebAssembly](https://webassembly.org/), giving you full control over performance.
- **Ergonomic and Intuitive**: Write code that feels natural. Everything is built on
  [reactive primitives](https://sycamore-rs.netlify.app/docs/basics/reactivity) without a cumbersome
  virtual DOM.
- **No JavaScript**: Had enough of JavaScript? So have we. Create apps using Sycamore without
  touching a single line of JS.

## Documentation

Sycamore is extensively documented:

- [Getting Started](https://sycamore-rs.netlify.app/docs/getting_started/installation): How to write
  your first Sycamore app.
- [Reactivity](https://sycamore-rs.netlify.app/docs/basics/reactivity): Find out how to use
  Sycamore's powerful reactive primitives.
- [API Documentation](https://docs.rs/sycamore): rustdocs for the `sycamore` crate.

**Still have questions?** Don't hesitate to stop by our friendly
[Discord server](https://discord.gg/vDwFUmm6mU).

## Examples

Sycamore has many examples for your reference in the
[`examples/`](https://github.com/sycamore-rs/sycamore/tree/master/examples) directory. Be sure to
cheek them out!

### Viewing on `sycamore-rs.netlify.app`

All the examples are hosted under `sycamore-rs.netlify.app/examples/<example_name>` with
`<example_name>` being the name of the example you want to view. For instance, the `hello` example
is hosted on
[`sycamore-rs.netlify.app/examples/hello`](https://sycamore-rs.netlify.app/examples/hello).

### Building Locally

All the examples can also be built locally using [Trunk](https://trunkrs.dev). For instance, the
following command builds and serves the `hello` example:

```bash
cd examples/hello
trunk serve
```

Now open up `localhost:8080` in your browser to see "Hello World!".

## Perseus

[Perseus](https://github.com/arctic-hen7/perseus) is a fullstack framework built with Sycamore. Think
[NextJS](https://nextjs.org) or [SvelteKit](https://kit.svelte.dev) but with no JavaScript.
Everything from backend to frontend is built with pure Rust!

## Alternatives?

Don't think Sycamore is for you? Thankfully, there are plenty of alternatives!
- **[SolidJS](https://github.com/solidjs/solid): A declarative, efficient and flexible JavaScript
  library for building user interfaces** <br />
  Solid is a JavaScript library which greatly inspired Sycamore. Many concepts such as fine-grained
  reactivity and components as factory functions were borrowed from Solid. If you don't mind working
  with JavaScript (or TypeScript), go check it out!
  
- **[Yew](https://github.com/yewstack/yew): Rust / Wasm framework for building client web apps** <br />
  Yew was also a big inspiration for Sycamore. Yew employs a VDOM and has a MVU (Elm) architecture. If
  you think that's for you, take a look!
  
- **[MoonZoon](https://github.com/MoonZoon/MoonZoon): Rust Fullstack Framework** <br />
  MoonZoon also champions the no VDOM paradigm and uses [dominator](https://github.com/Pauan/rust-dominator)
  as its underlying DOM layer. MoonZoon is a fullstack framework making it easier to combine frontend
  and backend code with minimal boilerplate.

## Contributing

- Report issues on our [issue tracker](https://github.com/sycamore-rs/sycamore/issues).
- We love Pull Requests! For more information, check out the
  [section on contributing](https://sycamore-rs.netlify.app/docs/contribute/architecture) in the
  docs.

Sycamore would not have been possible without the wonderful contributions from the community. Thank
you!

<a href="https://github.com/sycamore-rs/sycamore/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=sycamore-rs/sycamore" />
</a>
