use sycamore::prelude::*;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let name = create_signal(cx, String::new());
    let is_empty = create_selector(cx, || !name.get().is_empty());

    view! { cx,
        div {
            p {
                "Hello "
                (if *is_empty.get() {
                    view! { cx,
                        span { (name.get()) }
                    }
                } else {
                    view! { cx, span { "World" } }
                })
                "!"
            }
            input(bind:value=name)
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let s = sycamore::render_to_string(App);
    log::info!("{s}");

    sycamore::render(App);
}
