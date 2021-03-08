use maple_core::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let counter = Signal::new(0);

    create_effect({
        let counter = counter.clone();
        move || {
            log::info!("Counter value: {}", *counter.get());
        }
    });

    let increment = {
        let counter = counter.clone();
        move |_| counter.set(*counter.get() + 1)
    };

    let reset = {
        let counter = counter.clone();
        move |_| counter.set(0)
    };

    let root = template! {
        div {
            # "Counter demo"
            p(class="value") {
                # "Value: "
                # counter.get()
            }
            button(class="increment", on:click=increment) {
                # "Increment"
            }
            button(class="reset", on:click=reset) {
                # "Reset"
            }
        }
    };

    render(root);
}
