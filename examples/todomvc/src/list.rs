use sycamore::prelude::*;
use sycamore::context::use_context;

use crate::{
    AppState,
    Filter,
    item::Item,
};

#[component(List<G>)]
pub fn list() -> Template<G> {
    let app_state = use_context::<AppState>();
    let todos_left = create_selector(cloned!((app_state) => move || {
        app_state.todos_left()
    }));

    let filtered_todos = create_memo(cloned!((app_state) => move || {
        app_state.todos.get().iter().filter(|todo| match *app_state.filter.get() {
            Filter::All => true,
            Filter::Active => !todo.get().completed,
            Filter::Completed => todo.get().completed,
        }).cloned().collect::<Vec<_>>()
    }));

    // We need a separate signal for checked because clicking the checkbox will detach the binding
    // between the attribute and the view.
    let checked = Signal::new(false);
    create_effect(cloned!((checked) => move || {
        // Calling checked.set will also update the `checked` property on the input element.
        checked.set(*todos_left.get() == 0)
    }));

    template! {
        section(class="main") {
            input(
                id="toggle-all",
                class="toggle-all",
                type="checkbox",
                readonly=true,
                bind:checked=checked,
                on:input=cloned!((app_state) => move |_| app_state.toggle_complete_all())
            )
            label(for="toggle-all")

            ul(class="todo-list") {
                Keyed(KeyedProps {
                    iterable: filtered_todos,
                    template: move |todo| template! {
                        Item(todo)
                    },
                    key: |todo| todo.get().id,
                })
            }
        }
    }
}

