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

