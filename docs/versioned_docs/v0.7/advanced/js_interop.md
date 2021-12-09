# JS Interop

Sycamore is build upon [`wasm-bindgen`](https://rustwasm.github.io/wasm-bindgen/) which allows
calling JS and browser APIs from Rust. This means that you can also use `wasm-bindgen` to call JS
from Rust. However, for calling browser APIs, the `web-sys` and `js-sys` crates have automatically
generated bindings.

For more information, checkout the
[`wasm-bindgen` book](https://rustwasm.github.io/docs/wasm-bindgen/).
