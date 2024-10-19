---
title: Your First App
---

# Creating your first Sycamore app

This section will now guide you through creating your first Sycamore app.

## Prerequisites

### Install Rust

First, you'll need to install Rust. If you have already done this, you can skip
to the next section. Follow the
[official instructions](https://www.rust-lang.org/tools/install) to get started.
It is strongly recommended to use the `rustup` tool to manage your Rust
installation as it makes it very easy to install additional targets and
toolchains. In fact, we will use it right now to install the
`wasm32-unknown-unknown` target, which is needed to compile Rust to WebAssembly.

```bash
rustup target add wasm32-unknown-unknown
```

The minimum supported Rust toolchain is `v1.81.0`. Sycamore is not guaranteed to
(and probably won't) compile on older versions of Rust.

Sycamore only works on Rust edition 2021 or later. Even though most crates
written in edition 2021 are backward compatible with older editions, this is not
the case for Sycamore because Sycamore's proc-macro generates code that is only
compatible with edition 2021. Furthermore, the
[disjoint capture in closures](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html#disjoint-capture-in-closures)
feature greatly improves the ergonomics when working with Sycamore's reactivity.

### Install Trunk

[Trunk](https://trunkrs.dev) is the recommended build tool for Sycamore. If you
are familiar with JavaScript frontend development, Trunk is like
[webpack](https://webpack.js.org/) or [rollup](https://rollupjs.org/) but with
first-class support for building Rust WASM apps.

> We are currently working on building a Sycamore specific CLI tool but it is
> not yet ready. For now, Trunk is the recommended to build your Sycamore app.

You can use one of the following command to install Trunk on your system:

```bash
# Install via homebrew on Mac, Linux or Windows (WSL).
brew install trunk

# Install a release binary (great for CI).
# You will need to specify a value for ${VERSION}.
wget -qO- https://github.com/thedodd/trunk/releases/download/${VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-

# Install via cargo.
cargo install --locked trunk
```

For more information, check out the [Trunk website](https://trunkrs.dev)

## Create a new Sycamore project

Now with all the prerequisites installed, we can create our new Sycmore app.

```bash
cargo new hello-sycamore
cd hello-sycamore
```

This creates a blank Rust project called `hello-sycamore`. Let's add Sycamore as
a dependency:

```bash
cargo add sycamore@0.9.0-beta.4
```

You can also do this manually by adding the following line to your `Cargo.toml`
file.

```toml
sycamore = "0.9.0-beta.4"
```

### Hello, world!

Open up your `src/main.rs` file and replace the contents with:

```rust
use sycamore::prelude::*;

fn main() {
    sycamore::render(|| "Hello, world!".into());
}
```

Finally, create a new `index.html` file at the root of your project directory
with the following contents:

```html
<!DOCTYPE html>
<html>
    <head></head>
    <body></body>
</head>
```

This is required for Trunk to build your app to WebAssembly.

All that's left to do is to run:

```bash
trunk serve
```

This will build your app and automatically rebuild it whenever you make a change
to your source file, which is very useful for development. Open up
<http://localhost:8080> in your web browser. If you see the words "Hello,
world!" on the screen, then congratulations! You have successfully created your
first Sycamore app!
