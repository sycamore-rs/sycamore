use maple_core::prelude::*;

use crate::AppState;

pub fn List(app_state: AppState) -> TemplateResult {
    template! {
        section(class="main") {
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
