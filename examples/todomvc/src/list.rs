use maple_core::prelude::*;
use web_sys::HtmlInputElement;

use crate::{AppState, Filter};

#[component(List<G>)]
pub fn list(app_state: AppState) -> TemplateResult<G> {
    let todos_left = create_selector(cloned!((app_state) => move || {
        app_state.todos_left()
    }));

    let input_ref = NodeRef::<G>::new();

    // FIXME: bind to boolean attribute
    create_effect(cloned!((todos_left, input_ref) => move || {
        let checked = *todos_left.get() == 0;

        if let Some(input_ref) = input_ref.try_get::<DomNode>() {
            input_ref.unchecked_into::<HtmlInputElement>().set_checked(checked);
        }
    }));

    let filtered_todos = create_memo(cloned!((app_state) => move || {
        app_state.todos.get().iter().filter(|todo| match *app_state.filter.get() {
            Filter::All => true,
            Filter::Active => !todo.get().completed,
            Filter::Completed => todo.get().completed,
        }).cloned().collect::<Vec<_>>()
    }));

    template! {
        section(class="main") {
            input(
                ref=input_ref,
                id="toggle-all",
                class="toggle-all",
                type="checkbox",
                readonly=true,
                on:input=cloned!((app_state) => move |_| app_state.toggle_complete_all())
            )
            label(for="toggle-all")

            ul(class="todo-list") {
                Keyed<_, _, _, _>(KeyedProps {
                    iterable: filtered_todos,
                    template: move |todo| template! {
                        crate::item::Item(crate::item::ItemProps { todo, app_state: app_state.clone() })
                    },
                    key: |todo| todo.get().id,
                })
            }
        }
    }
}
