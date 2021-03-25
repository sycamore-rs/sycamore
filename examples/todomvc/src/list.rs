use maple_core::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use crate::AppState;

pub fn List(app_state: AppState) -> TemplateResult {
    let todos_left = create_selector(cloned!((app_state) => move || {
        app_state.todos_left()
    }));

    let input_ref = NodeRef::new();

    // FIXME: bind to boolean attribute
    create_effect(cloned!((todos_left, input_ref) => move || {
        let checked = *todos_left.get() == 0;

        if let Some(input_ref) = input_ref.try_get() {
            input_ref.unchecked_into::<HtmlInputElement>().set_checked(checked);
        }
    }));

    template! {
        section(class="main") {
            input(
                ref=input_ref,
                id="toggle-all",
                class="toggle-all",
                type="checkbox",
                // checked=*todos_left.get() == 0,
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
                    key: |todo| todo.get().id,
                })
            }
        }
    }
}
