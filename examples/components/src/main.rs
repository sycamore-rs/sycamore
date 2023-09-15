use sycamore::prelude::*;

#[component(inline_props)]
fn MyComponent<G: Html>(value: Signal<i32>) -> View<G> {
    view! {
        div(class="my-component") {
            "My component"
            p {
                "Value: "
                (value.get())
            }
        }
    }
}

#[component]
fn App<G: Html>() -> View<G> {
    let mut state = create_signal(0);

    let increment = move |_| state += 1;

    view! {
        div {
            "Component demo"

            MyComponent(value=state)
            MyComponent(value=state)

            button(on:click=increment) { "+" }
        }
    }
}

fn main() {
    sycamore::render(App);
}
