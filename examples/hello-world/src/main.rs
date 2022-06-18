use sycamore::prelude::*;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        p {
            "Hello World!"
        }
    }
}

fn main() {
    sycamore::render(|cx| {
        view! { cx, App() }
    });
}
