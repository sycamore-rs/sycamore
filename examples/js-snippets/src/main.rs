use sycamore::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/js/index.js")]
extern "C" {
    fn multiply(a: f64, b: f64) -> f64;
}

#[component]
fn App(cx: Scope) -> View {
    let a = create_signal(cx, 0_f64);
    let b = create_signal(cx, 0_f64);
    let product = create_signal(cx, 0_f64);

    create_effect(cx, || {
        product.set(multiply(*a.get(), *b.get()));
    });

    view! { cx,
        input(_type="number", bind:value_as_number=a)
        span { "*" }
        input(_type="number", bind:value_as_number=b)
        span { "=" }
        span { (*product.get()) }
    }
}

fn main() {
    sycamore::render(App);
}
