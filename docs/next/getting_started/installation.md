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

### Minimum Supported Rust Version (MSRV)

The minimum supported Rust toolchain is `v1.53.0`. Sycamore is not guaranteed to compile on an older
version of Rust.

## Install Trunk

[Trunk](https://trunkrs.dev) is the recommended build tool for Sycamore. If you are from JS land,
Trunk is like [webpack](https://webpack.js.org/) or [rollup](https://rollupjs.org/) but specifically
tailored towards Rust + Wasm apps.

You can use the following command to install Trunk:

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

You now need to add Sycamore to your new project's dependencies. Add the following to your
`Cargo.toml` file in your project folder:

```toml
sycamore = "0.6.3"
```

> Note: Sycamore is currently being developed at a rapid pace. To have access to the latest
> features, consider using a
> [git dependency](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories)
> instead.
>
> However, breaking changes are introduced all the time. It is therefore recommended to also specify
> a specific commit hash to prevent your code from randomly breaking. You can find the latest commit
> hash [here](https://github.com/sycamore-rs/sycamore/commits/master) (to the right of each commit).
>
> ```toml
> # Make sure you update the rev to the latest commit hash.
> sycamore = { git = "https://github.com/sycamore-rs/sycamore", rev = "fc640d313e66f9a6af422fae44f4f72fa86280cc" }
> ```

You should now be all set for your Sycamore adventure!
