use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> View<G> {
    let items = Signal::new(vec![
        view! { "Hello!" },
        view! { "I am an item in a fragment"},
    ]);

    let add_item = cloned!((items) => move |_| {
        items.set(
            (*items.get())
                .clone()
                .into_iter()
                .chain(Some(view! { "New item" }))
                .collect(),
        );
    });

    view! {
        div {
            button(on:click=add_item) { "Add item" }
            div(class="items") {
                (View::new_fragment((*items.get()).clone()))
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| view! { App() });
}
