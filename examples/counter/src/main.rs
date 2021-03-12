#![allow(non_snake_case)]

use maple_core::prelude::*;

fn App() -> TemplateResult {
    let counter = Signal::new(0);

    create_effect(cloned!((counter) => move || {
        log::info!("Counter value: {}", *counter.get());
    }));

    let increment = cloned!((counter) => move |_| counter.set(*counter.get() + 1));

    let reset = cloned!((counter) => move |_| counter.set(0));

    template! {
        div {
            "Counter demo"
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

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
