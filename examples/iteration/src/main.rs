use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> Template<G> {
    let (items, set_items) = create_signal(vec![
        template! { "Hello!" },
        template! { "I am an item in a fragment"},
    ]);

    let add_item = move |_| {
        set_items.set(
            (*items.get())
                .clone()
                .into_iter()
                .chain(Some(template! { "New item" }))
                .collect(),
        );
    };

    template! {
        div {
            button(on:click=add_item) { "Add item" }
            div(class="items") {
                (Template::new_fragment((*items.get()).clone()))
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| template! { App() });
}
