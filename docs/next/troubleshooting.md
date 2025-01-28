---
title: Troubleshooting
---

# Troubleshooting

## Viewing panic messages in the browser

By default, Rust does not print panic messages to the console on
`wasm32-unknown-unknown`. To see panic messages, add the
[`console_error_panic_hook`](https://docs.rs/console_error_panic_hook/latest/console_error_panic_hook/)
crate and add the following line to your `fn main`:

```rust
console_error_panic_hook::set_once();
```

## Debugging using DWARF + WASM

> Note: This section is a stub. Help us write this section!

## unexpected `cfg` condition name: `sycamore_force_ssr`

Sycamore uses a custom cfg (`sycamore_force_ssr`) to force the SSR mode. Because
the compiler doesn't know about custom cfg it will emit warnings. To disables
those warnings, add the following lints configuration in the `Cargo.toml` file
of your project.

```toml
[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ["cfg(sycamore_force_ssr)"] }
```

More information here:
<https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html#check-cfg-in-lintsrust-table>
