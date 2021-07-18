# Getting Started

This documentation assumes the developer is already familiar with Rust programming. To learn more
about Rust, check out the [Rust Book](https://doc.rust-lang.org/stable/book/).

## Install Rust

First, you'll need to install Rust. Follow the
[official instructions](https://www.rust-lang.org/tools/install) to get started.

You will also need the `wasm32-unknown-unknown` target installed:

```bash
rustup target add wasm32-unknown-unknown
```

## Install Trunk

[Trunk](https://trunkrs.dev) is the recommended build tool for Sycamore. If you are from JS land,
Trunk is like [webpack](https://webpack.js.org/) or [rollup](https://rollupjs.org/) but specifically
tailored to Rust + WASM apps.

You can use `cargo` to install Trunk:

```bash
cargo install trunk wasm-bindgen-cli
```

For more information, head over to the [Trunk website](https://trunkrs.dev)

## Create a new Sycamore project

Create a new Rust project using `cargo`:

```bash
cargo new my-project
cd my-project
```

You now need to add Sycamore to your new project's dependencies. Add the following to your
`Cargo.toml` file in your project folder:

```toml
sycamore = ""
```

You should now be all set for your Sycamore adventure!
