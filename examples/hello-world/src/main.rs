use sycamore::prelude::*;

#[component]
fn App<G: Html>() -> View<G> {
    view! {
        p {
            "Hello World!"
        }
    }
}

fn main() {
    sycamore::render(App);
}
