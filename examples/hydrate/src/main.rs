use sycamore::prelude::*;

#[component(Counter<G>)]
fn counter() -> View<G> {
    let counter = Signal::new(0);

    let increment = cloned!((counter) => move |_| counter.set(*counter.get() + 1));

    let reset = cloned!((counter) => move |_| counter.set(0));

    view! {
        div {
            h2 { "Counter demo" }
            p(class="value") {
                "Value: "
                (counter.get())
            }
            button(class="increment", on:click=increment) {
                "Increment"
            }
            button(class="reset", on:click=reset) {
                "Reset"
            }
        }
    }
}

#[component(Hello<G>)]
fn hello() -> View<G> {
    let name = Signal::new(String::new());
    let name2 = name.clone();

    view! {
        div {
            h2 {
                "Hello "
                (if *create_selector(cloned!((name) => move || !name.get().is_empty())).get() {
                    cloned!((name) => view! {
                        span { (name.get()) }
                    })
                } else {
                    view! { span { "World" } }
                })
                "!"
            }

            input(bind:value=name2)
        }
    }
}

#[component(App<G>)]
fn app() -> View<G> {
    view! {
        h1 { "Hydration example" }
        Hello()
        Counter()
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let s = sycamore::render_to_string(|| view! { App() });
    log::info!("{}", s);

    sycamore::hydrate(|| view! { App() });
}
