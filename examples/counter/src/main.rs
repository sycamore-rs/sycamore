use sycamore::prelude::*;

#[component]
fn App<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let state = ctx.create_signal(0i32);
    let increment = |_| state.set(*state.get() + 1);
    let decrement = |_| state.set(*state.get() - 1);
    let reset = |_| state.set(0);
    view! { ctx,
        div {
            p { "Value: " (state.get()) }
            button(on:click=increment) { "+" }
            button(on:click=decrement) { "-" }
            button(on:click=reset) { "Reset" }
        }
    }
}

fn main() {
    sycamore::render(|ctx| {
        view! { ctx,
            App {}
        }
    });
}
