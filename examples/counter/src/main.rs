use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> View<G> {
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

    sycamore::render(|| template! { App() });
}
