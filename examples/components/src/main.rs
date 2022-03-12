use sycamore::prelude::*;

#[component]
fn MyComponent<'a, G: Html>(ctx: Scope<'a>, props: &'a Signal<i32>) -> View<G> {
    view! { ctx,
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
fn App<G: Html>(ctx: Scope) -> View<G> {
    let state = ctx.create_signal(0);

    let increment = |_| state.set(*state.get() + 1);

    view! { ctx,
        div {
            "Component demo"

            MyComponent(state)
            MyComponent(state)

            button(on:click=increment) { "+" }
        }
    }
}

fn main() {
    sycamore::render(|ctx| view! { ctx, App() });
}
