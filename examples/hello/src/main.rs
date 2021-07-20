use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> Template<G> {
    let (name, set_name) = create_signal(String::new());

    template! {
        div {
            h1 {
                "Hello "
                (if *create_selector(move || !name.get().is_empty()).get() {
                    template! {
                        span { (name.get()) }
                    }
                } else {
                    template! { span { "World" } }
                })
                "!"
            }

            input(bind:value=(name, set_name))
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| template! { App() });
}
