use maple_core::prelude::*;

pub fn Usage() -> TemplateResult {
    template! {
        div(class="container") {
            h1 { "Maple" }
            p { "A reactive DOM library for Rust in WASM" }
        }
    }
}
