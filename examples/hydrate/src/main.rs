use sycamore::prelude::*;

#[component]
fn Counter<G: Html>(cx: Scope) -> View<G> {
    let state = create_signal(cx, 0i32);
    let increment = |_| state.set(*state.get() + 1);
    let decrement = |_| state.set(*state.get() - 1);
    let reset = |_| state.set(0);
    view! { cx,
        div {
            p { "Value: " (state.get()) }
            button(on:click=increment) { "+" }
            button(on:click=decrement) { "-" }
            button(on:click=reset) { "Reset" }
        }
    }
}

#[component]
fn Hello<G: Html>(cx: Scope) -> View<G> {
    let name = create_signal(cx, String::new());

    view! { cx,
        div {
            p {
                "Hello "
                (if *create_selector(cx, || !name.get().is_empty()).get() {
                    view! { cx,
                        span { (name.get()) }
                    }
                } else {
                    view! { cx, span { "World" } }
                })
                "!"
            }
            input(bind:value=name)
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        p { "Hydration" }
        br
        Hello {}
        br
        Counter {}

        sycamore::web::NoHydrate {
            p { "This paragraph is not hydrated!" }
        }
        sycamore::web::NoSsr {
            p { "This paragraph is only rendered on the client side" }
        }
    }
}
fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let s = sycamore::render_to_string(|cx| view! { cx, App {} });
    log::info!("{}", s);

    sycamore::hydrate(|cx| view! { cx, App {} });
}
