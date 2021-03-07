use maple_core::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let (counter, set_counter) = create_signal(0);

    create_effect({
        let counter = counter.clone();
        move || {
            log::info!("Counter value: {}", *counter());
        }
    });

    let increment = {
        let counter = counter.clone();
        let set_counter = set_counter.clone();

        move || set_counter(*counter() + 1)
    };

    let reset = {
        let set_counter = set_counter.clone();

        move || set_counter(0)
    };

    let root = template! {
        div {
            # "Counter demo"
            p(class="value") {
                # "Value: "
                # counter()
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
