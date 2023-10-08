use sycamore::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/js/index.js")]
extern "C" {
    fn multiply(a: f64, b: f64) -> f64;
}

#[component]
fn App<G: Html>() -> View<G> {
    let a = create_signal(1_f64);
    let b = create_signal(1_f64);
    let product = create_signal(1_f64);

    create_effect(move || {
        product.set(multiply(a.get(), b.get()));
    });

    view! {
        input(type="number", bind:valueAsNumber=a)
        span { "*" }
        input(type="number", bind:valueAsNumber=b)
        span { "=" }
        span { (product.get()) }
    }
}

fn main() {
    sycamore::render(App);
}
