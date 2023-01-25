use sycamore::prelude::*;

#[component(inline_props)]
fn MyComponent<'a>(cx: Scope<'a>, value: &'a Signal<i32>) -> View {
    view! { cx,
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
fn App(cx: Scope) -> View {
    let state = create_signal(cx, 0);

    let increment = |_| state.set(*state.get() + 1);

    view! { cx,
        div {
            "Component demo"

            MyComponent(value=state)
            MyComponent(value=state)

            button(on:click=increment) { "+" }
        }
    }
}

fn main() {
    sycamore::render(|cx| view! { cx, App {} });
}
