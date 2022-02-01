use sycamore::prelude::*;

#[component]
fn Counter<G: Html>(ctx: ScopeRef) -> View<G> {
    let counter = ctx.use_context::<RcSignal<i32>>();

    view! { ctx,
        p(class="value") {
            "Value: " (counter.get())
        }
    }
}

#[component]
pub fn Controls<G: Html>(ctx: ScopeRef) -> View<G> {
    let state = ctx.use_context::<RcSignal<i32>>();
    let increment = move |_| state.set(*state.get() + 1);
    let decrement = move |_| state.set(*state.get() - 1);
    let reset = move |_| state.set(0);

    view! { ctx,
        button(on:click=decrement) { "-" }
        button(on:click=increment) { "+" }
        button(on:click=reset) { "Reset" }
    }
}

#[component]
fn App<G: Html>(ctx: ScopeRef) -> View<G> {
    let counter = create_rc_signal(0i32);
    ctx.provide_context(counter);

    view! { ctx,
        div {
            "Context demo"
            Counter()
            Controls()
        }
    }
}

fn main() {
    sycamore::render(|ctx| view! { ctx, App() });
}
