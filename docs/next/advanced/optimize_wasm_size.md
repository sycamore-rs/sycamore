# Reducing binary size

**Note**: More information about reducing binary size can be found in the
[Rust Wasm Book](https://rustwasm.github.io/book/reference/code-size.html#optimizing-builds-for-code-size)

## Building in release mode

A common mistake when building a Wasm binary is to forget to build in release mode. If you are using
`trunk`, simply add the `--release` flag to the build command:

```bash
trunk build --release
```

## `wee_alloc`

[`wee_alloc`](https://github.com/rustwasm/wee_alloc) is a memory allocator focused on targeting
WebAssembly, producing a small .wasm code size, and having a simple, correct implementation.
Replacing the default allocator with this one will result in a smaller binary size at the expense of
speed and memory overhead.

```rust
// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

## `Cargo.toml`

It is possible to configure release builds to be smaller by using various flags and configurations
in your `Cargo.toml` file.

```toml
[profile.release]
# Do not perform backtrace for panic on release builds.
panic = 'abort'
# Perform optimizations on all codegen units.
codegen-units = 1
# Optimize for size.
opt-level = 's' # or 'z' to optimize "aggressively" for size
# Enable link time optimization.
lto = true
```

## `wasm-opt`

If you are using `trunk`, add this line to your `index.html` to enable `wasm-opt`:

```html
<link data-trunk rel="rust" data-wasm-opt="s" />
```
