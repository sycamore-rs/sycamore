use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> Template<G> {
    let (counter, set_counter) = create_signal(0);

    create_effect(move || {
        log::info!("Counter value: {}", *counter.get());
    });

    let increment = move |_| set_counter.set(*counter.get() + 1);
    let reset = move |_| set_counter.set(0);

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
