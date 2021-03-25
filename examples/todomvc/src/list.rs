use maple_core::prelude::*;

use crate::AppState;

pub fn List(app_state: AppState) -> TemplateResult {
    let todos_left = create_selector(cloned!((app_state) => move || {
        app_state.todos.get();
        app_state.todos_left()
    }));

    template! {
        section(class="main") {
            input(
                id="toggle-all",
                class="toggle-all",
                type="checkbox",
                checked=*todos_left.get() == 0,
                readonly=true,
                on:input=cloned!((app_state) => move |_| app_state.toggle_complete_all())
            )
            label(for="toggle-all")

            ul(class="todo-list") {
                Keyed(KeyedProps {
                    iterable: app_state.todos.clone(),
                    template: move |todo| template! {
                        crate::item::Item(todo, app_state.clone())
                    },
                    key: |todo| todo.id,
                })
            }
        }
    }
}
