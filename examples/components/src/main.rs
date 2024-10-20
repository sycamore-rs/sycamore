use sycamore::prelude::*;

#[component(inline_props)]
fn MyComponent(value: ReadSignal<i32>) -> View {
    view! {
        div(class="my-component") {
            "My component"
            p {
                "Value: " (value)
            }
        }
    }
}

#[component]
fn App() -> View {
    let mut state = create_signal(0);

    let increment = move |_| state += 1;

    view! {
        div {
            "Component demo"

            MyComponent(value=*state)
            MyComponent(value=*state)

            button(on:click=increment) { "+" }
        }
    }
}

fn main() {
    sycamore::render(App);
}
