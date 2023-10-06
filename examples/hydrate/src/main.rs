use sycamore::prelude::*;

#[component]
fn Counter<G: Html>() -> View<G> {
    let mut state = create_signal(0i32);
    let increment = move |_| state += 1;
    let decrement = move |_| state -= 1;
    let reset = move |_| state.set(0);
    view! {
        div {
            p { "Value: " (state.get()) }
            button(on:click=increment) { "+" }
            button(on:click=decrement) { "-" }
            button(on:click=reset) { "Reset" }
        }
    }
}

#[component]
fn Hello<G: Html>() -> View<G> {
    let name = create_signal(String::new());
    let is_empty = create_selector(move || !name.with(String::is_empty));

    view! {
        div {
            p {
                "Hello "
                (if is_empty.get() {
                    view! {
                        span { (name.get_clone()) }
                    }
                } else {
                    view! { span { "World" } }
                })
                "!"
            }
            input(bind:value=name)
        }
    }
}

#[component]
fn App<G: Html>() -> View<G> {
    view! {
        p { "Hydration" }
        br {}
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

    let s = sycamore::render_to_string(App);
    log::info!("{}", s);

    sycamore::hydrate(App);
}
