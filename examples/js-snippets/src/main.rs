use sycamore::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/js/functions.js")]
extern "C" {
    fn multiply(a: f64, b: f64) -> f64;
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let a = create_signal(cx, 0_f64);
    let b = create_signal(cx, 0_f64);
    let product = create_signal(cx, 0_f64);

    create_effect(cx, || {
        product.set(multiply(*a.get(), *b.get()));
    });

    view! { cx,
        input(type="number", bind:valueAsNumber=a)
        span {"*"}
        input(type="number", bind:valueAsNumber=b)
        span {"="}
        span {(*product.get())}
    }
}

fn main() {
    sycamore::render(|cx| {
        view! { cx,
            App
        }
    });
}
