use sycamore::prelude::*;

#[component]
fn MyComponent<'a, G: Html>(cx: Scope<'a>, props: &'a Signal<i32>) -> View<G> {
    view! { cx,
        div(class="my-component") {
            "My component"
            p {
                "Value: "
                (props.get())
            }
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let state = cx.create_signal(0);

    let increment = |_| state.set(*state.get() + 1);

    view! { cx,
        div {
            "Component demo"

            MyComponent(state)
            MyComponent(state)

            button(on:click=increment) { "+" }
        }
    }
}

fn main() {
    sycamore::render(|cx| view! { cx, App() });
}
