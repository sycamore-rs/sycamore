---
title: Optimize WASM Size
---

# Reducing binary size

**Note**: More information about reducing binary size can be found in the
[Rust Wasm Book](https://rustwasm.github.io/book/reference/code-size.html#optimizing-builds-for-code-size).

## Building in release mode

If you are building for production, make sure you are serving a release WASM
binary. Rust tends to produce very large WASM binaries during debug mode but
simply passing the `--release` will massively reduce the total payload.

```bash
trunk build --release
```

## `Cargo.toml` flags

It is possible to configure release builds to be smaller by using various flags
and configurations in your `Cargo.toml` file.

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

## Using `wasm-opt`

If you are using `trunk`, add this line to your `index.html` to enable
`wasm-opt`:

```html
<link data-trunk rel="rust" data-wasm-opt="s" />
```
