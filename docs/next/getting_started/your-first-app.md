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

### Minimum Supported Rust Version (MSRV) and Rust edition

The minimum supported Rust toolchain is `v1.81.0`. Sycamore is not guaranteed to
(and probably won't) compile on older versions of Rust.

Sycamore only works on Rust edition 2021. Even though most crates written in
edition 2021 are backward compatible with older editions, this is not the case
for Sycamore because Sycamore's proc-macro generates code that is only
compatible with edition 2021. Furthermore, the
[disjoint capture in closures](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html#disjoint-capture-in-closures)
feature greatly improves the ergonomics when working with Sycamore's reactivity.

## Install Trunk

[Trunk](https://trunkrs.dev) is the recommended build tool for Sycamore. If you
are from JS land, Trunk is like [webpack](https://webpack.js.org/) or
[rollup](https://rollupjs.org/) but specifically tailored towards Rust + WASM
apps.

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

For more information, head over to the [Trunk website](https://trunkrs.dev)

## Create a new Sycamore project

Create a new Rust project using `cargo`:

```bash
cargo new my-project
cd my-project
```

You now need to add Sycamore to your new project's dependencies. Add the
following to your `Cargo.toml` file in your project folder:

```toml
sycamore = "0.9.0-beta.4"
```

> **Note**: Sycamore is currently being developed at a rapid pace. To have
> access to the latest features, consider using a
> [git dependency](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories)
> instead.
>
> However, breaking changes are introduced all the time. It is therefore
> recommended to also specify a specific commit hash to prevent your code from
> randomly breaking. You can find the latest commit hash
> [here](https://github.com/sycamore-rs/sycamore/commits/main) (to the right of
> each commit).
>
> ```toml
> # Make sure you update the rev to the latest commit hash.
> sycamore = { git = "https://github.com/sycamore-rs/sycamore", rev = "c5b4097" }
> ```

You should now be all set for your Sycamore adventure!
