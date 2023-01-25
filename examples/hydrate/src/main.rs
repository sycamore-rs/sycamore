use sycamore::prelude::*;
use sycamore::web::document;

#[component]
fn Counter(cx: Scope) -> View {
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
fn Hello(cx: Scope) -> View {
    let name = create_signal(cx, String::new());
    let is_not_empty = create_selector(cx, || !name.get().is_empty());

    view! { cx,
        div {
            p {
                "Hello "
                (if *is_not_empty.get() {
                    view! { cx, span { (name.get()) } }
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
fn App(cx: Scope) -> View {
    view! { cx,
        p { "Hydration" }
        Hello {}
        br {}
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

    let ssr = sycamore::render_to_string(|cx| view! { cx, App {} });
    log::warn!("We are rendering the app to a string directly in the browser.
Generally, for a real life app, this would make no sense.
However, we are doing this here for demonstration purposes without needing to set up a full-blown server.

If you ever change the code in this example, make sure to update the index.html file with the string below.");
    log::info!("{}", ssr);

    let root = document().get_element_by_id("main").unwrap();
    sycamore::hydrate_to(&root, |cx| view! { cx, App {} });
}
