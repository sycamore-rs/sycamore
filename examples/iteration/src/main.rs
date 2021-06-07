use maple_core::prelude::*;

#[component(App<G>)]
fn app() -> TemplateResult<G> {
    let items = Signal::new(vec![
        template! { "Hello!" },
        template! { "I am an item in a fragment"},
    ]);

    let add_item = cloned!((items) => move |_| {
        items.set(
            (*items.get())
                .clone()
                .into_iter()
                .chain(Some(template! { "New item" }))
                .collect(),
        );
    });

    template! {
        div {
            button(on:click=add_item) { "Add item" }
            div(class="items") {
                (TemplateResult::new_fragment((*items.get()).clone()))
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() App2() });
}
