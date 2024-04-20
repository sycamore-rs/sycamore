use sycamore::prelude::*;

#[component]
fn App() -> View {
    view! {
        p {
            "Hello World!"
        }
    }
}

fn main() {
    sycamore::render(App);
}
