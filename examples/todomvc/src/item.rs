use maple_core::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, KeyboardEvent};

use crate::{AppState, Todo};

pub fn Item(todo: Todo, app_state: AppState) -> TemplateResult {
    let task = todo.task.clone();
    let id = todo.id;

    let editing = Signal::new(false);
    let input_ref = NodeRef::new();
    let value = Signal::new("".to_string());

    let handle_input = cloned!((value) => move |event: Event| {
        let target: HtmlInputElement = event.target().unwrap().unchecked_into();
        value.set(target.value());
    });

    let toggle_completed = cloned!((app_state) => move |_| {
        app_state.toggle_completed(id);
    });

    let handle_dblclick = cloned!((editing, input_ref) => move |_| {
        editing.set(true);
        input_ref.get().unchecked_into::<HtmlInputElement>().focus().unwrap();
    });

    let handle_blur = cloned!((app_state, editing, value) => move || {
        editing.set(false);

        let mut value = value.get().as_ref().clone();
        value = value.trim().to_string();

        if value.is_empty() {
            app_state.remove_todo(id);
        } else {
            app_state.edit_todo_task(id, value);
        }
    });

    let handle_submit = cloned!((editing, input_ref, handle_blur, task) => move |event: Event| {
        let event: KeyboardEvent = event.unchecked_into();
        match event.key().as_str() {
            "Enter" => handle_blur(),
            "Escape" => {
                input_ref.get().unchecked_into::<HtmlInputElement>().set_value(&task);
                editing.set(false);
            },
            _ => {}
        }
    });

    let handle_destroy = cloned!((app_state) => move |_| {
        app_state.remove_todo(id);
    });

    let completed = todo.completed;

    let class = cloned!((editing) => move || {
        format!("{} {}",
            if completed { "completed" } else { "" },
            if *editing.get() { "editing" } else { "" }
        )
    });

    template! {
        li(class=class()) {
            div(class="view") {
                input(class="toggle", type="checkbox", checked=completed, on:input=toggle_completed)
                label(on:dblclick=handle_dblclick) {
                    (task.clone())
                }
                button(class="destroy", on:click=handle_destroy)
            }

            (if *editing.get() {
                cloned!((todo, input_ref, handle_blur, handle_submit, handle_input) => template! {
                    input(ref=input_ref,
                        class="edit",
                        value=todo.task.clone(),
                        on:blur=move |_| handle_blur(),
                        on:keyup=handle_submit,
                        on:input=handle_input,
                    )
                })
            } else {
                TemplateResult::empty()
            })
        }
    }
}
