use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> View<G> {
    let name = Signal::new(String::new());
    let name2 = name.clone();

    view! {
        div {
            h1 {
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

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| view! { App() });
}
