use sycamore::prelude::*;

#[component]
fn App(cx: Scope) -> View {
    view! { cx,
        p {
            "Hello World!"
        }
    }
}

fn main() {
    sycamore::render(|cx| {
        view! { cx, App {} }
    });
}
