use sycamore::builder::html::*;
use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> View<G> {
    let name = Signal::new(String::new());

    div()
        .child(
            h1().text("Hello ")
                .dyn_child(cloned!((name) => move || {
                    if *create_selector(cloned!((name) => move || !name.get().is_empty())).get() {
                        span()
                            .dyn_text(cloned!((name) => move || name.get().to_string()))
                            .build()
                    } else {
                        span().text("World").build()
                    }
                }))
                .text("!")
                .build(),
        )
        .child(input().bind_value(name).build())
        .build()
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| view! { App() });
}
