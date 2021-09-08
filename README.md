# Sycamore

[![Crates.io](https://img.shields.io/crates/v/sycamore)](https://crates.io/crates/sycamore)
[![docs.rs](https://img.shields.io/docsrs/sycamore?color=blue&label=docs.rs)](https://docs.rs/sycamore)
[![GitHub contributors](https://img.shields.io/github/contributors/sycamore-rs/sycamore)](https://github.com/sycamore-rs/sycamore/graphs/contributors)
[![Discord](https://img.shields.io/discord/820400041332179004?label=discord)](https://discord.gg/vDwFUmm6mU)

## What is Sycamore?

Sycamore is a modern VDOM-less web library with fine-grained reactivity in
[Rust](https://www.rust-lang.org/) and [WebAssembly](https://webassembly.org/).

- **Lightning Speed**: Sycamore harnesses the full power of [Rust](https://www.rust-lang.org/) via
  WebAssembly(https://webassembly.org/), giving you full control over performance.
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
```
