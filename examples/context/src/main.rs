use sycamore::prelude::*;

#[component]
fn Counter<G: Html>() -> View<G> {
    let counter = use_context::<Signal<i32>>();

    view! {
        p(class="value") {
            "Value: " (counter.get())
        }
    }
}

#[component]
pub fn Controls<G: Html>() -> View<G> {
    let mut state = use_context::<Signal<i32>>();
    let increment = move |_| state += 1;
    let decrement = move |_| state -= 1;
    let reset = move |_| state.set(0);

    view! {
        button(on:click=decrement) { "-" }
        button(on:click=increment) { "+" }
        button(on:click=reset) { "Reset" }
    }
}

#[component]
fn App<G: Html>() -> View<G> {
    let counter = create_signal(0i32);
    provide_context(counter);

    view! {
        div {
            "Context demo"
            Counter {}
            Controls {}
        }
    }
}

fn main() {
    sycamore::render(App);
}
