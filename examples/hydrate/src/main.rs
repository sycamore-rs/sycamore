use sycamore::prelude::*;

#[component]
fn Counter<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
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

#[component]
fn Hello<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let name = ctx.create_signal(String::new());

    view! { ctx,
        div {
            p {
                "Hello "
                (if *ctx.create_selector(|| !name.get().is_empty()).get() {
                    view! { ctx,
                        span { (name.get()) }
                    }
                } else {
                    view! { ctx, span { "World" } }
                })
                "!"
            }
            input(bind:value=name)
        }
    }
}

#[component]
fn App<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    view! { ctx,
        p { "Hydration" }
        br
        Hello {}
        Counter {}
    }
}
fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let s = sycamore::render_to_string(|ctx| view! { ctx, App {} });
    log::info!("{}", s);

    sycamore::hydrate(|ctx| view! { ctx, App {} });
}
