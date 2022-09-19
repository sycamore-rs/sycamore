use sycamore::prelude::*;

#[component]
fn Counter<G: Html>(cx: Scope) -> View<G> {
    let counter = use_context::<RcSignal<i32>>(cx);

    view! { cx,
        p(class="value") {
            "Value: " (counter.get())
        }
    }
}

#[component]
pub fn Controls<G: Html>(cx: Scope) -> View<G> {
    let state = use_context::<RcSignal<i32>>(cx);
    let increment = move |_| state.set(*state.get() + 1);
    let decrement = move |_| state.set(*state.get() - 1);
    let reset = move |_| state.set(0);

    view! { cx,
        button(on:click=decrement) { "-" }
        button(on:click=increment) { "+" }
        button(on:click=reset) { "Reset" }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let counter = create_rc_signal(0i32);
    provide_context(cx, counter);

    view! { cx,
        div {
            "Context demo"
            Counter {}
            Controls {}
        }
    }
}

fn main() {
    sycamore::render(|cx| view! { cx, App {} });
}
