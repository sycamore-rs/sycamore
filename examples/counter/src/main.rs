use sycamore::prelude::*;

#[component]
fn App() -> View {
    let mut state = create_signal(0i32);
    let increment = move |_| state += 1;
    let decrement = move |_| state -= 1;
    let reset = move |_| state.set(0);
    view! {
        div {
            p { "Value: " (state) }
            button(on:click=increment) { "+" }
            button(on:click=decrement) { "-" }
            button(on:click=reset) { "Reset" }
        }
    }
}

fn main() {
    sycamore::render(App);
}
