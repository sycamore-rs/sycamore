use sycamore::context::{ContextProvider, ContextProviderProps};
use sycamore::prelude::*;
use sycamore::reactive::use_context;

#[component(Counter<G>)]
fn counter() -> View<G> {
    let counter = use_context::<Signal<i32>>();

    template! {
        p(class="value") {
            "Value: "
            (counter.get())
        }
    }
}

#[component(Controls<G>)]
pub fn controls() -> View<G> {
    let counter = use_context::<Signal<i32>>();

    let increment = cloned!((counter) => move |_| counter.set(*counter.get() + 1));

    let reset = cloned!((counter) => move |_| counter.set(0));

    template! {
        button(class="increment", on:click=increment) {
            "Increment"
        }
        button(class="reset", on:click=reset) {
            "Reset"
        }
    }
}

#[component(App<G>)]
fn app() -> View<G> {
    let counter = Signal::new(0);

    create_effect(cloned!((counter) => move || {
        log::info!("Counter value: {}", *counter.get());
    }));

    template! {
        ContextProvider(ContextProviderProps {
            value: counter,
            children: move || {
                template! {
                    div {
                        "Counter demo"
                        Counter()
                        Controls()
                    }
                }
            }
        })
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| template! { App() });
}
