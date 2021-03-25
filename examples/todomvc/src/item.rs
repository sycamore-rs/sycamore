use maple_core::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, KeyboardEvent};

use crate::{AppState, Todo};

pub fn Item(todo: Signal<Todo>, app_state: AppState) -> TemplateResult {
    let task = cloned!((todo) => move || todo.get().task.clone());
    let id = todo.get().id;

    let editing = Signal::new(false);
    let input_ref = NodeRef::new();
    let value = Signal::new("".to_string());

    let handle_input = cloned!((value) => move |event: Event| {
        let target: HtmlInputElement = event.target().unwrap().unchecked_into();
        value.set(target.value());
    });

    let toggle_completed = cloned!((todo) => move |_| {
        todo.set(Todo {
            completed: !todo.get().completed,
            ..todo.get().as_ref().clone()
        });
    });

    let handle_dblclick = cloned!((editing, input_ref) => move |_| {
        editing.set(true);
        input_ref.get().unchecked_into::<HtmlInputElement>().focus().unwrap();
    });

    let handle_blur = cloned!((todo, app_state, editing, value) => move || {
        editing.set(false);

        let mut value = value.get().as_ref().clone();
        value = value.trim().to_string();

        if value.is_empty() {
            app_state.remove_todo(id);
        } else {
            todo.set(Todo {
                task: value,
                ..todo.get().as_ref().clone()
            })
        }
    });

    let handle_submit = cloned!((editing, input_ref, handle_blur, task) => move |event: Event| {
        let event: KeyboardEvent = event.unchecked_into();
        match event.key().as_str() {
            "Enter" => handle_blur(),
            "Escape" => {
                input_ref.get().unchecked_into::<HtmlInputElement>().set_value(&task());
                editing.set(false);
            },
            _ => {}
        }
    });

    let handle_destroy = cloned!((app_state) => move |_| {
        app_state.remove_todo(id);
    });

    let toggle_ref = NodeRef::new();

    // FIXME: bind to boolean attribute
    create_effect(cloned!((todo, toggle_ref) => move || {
        let completed = todo.get().completed;
        if let Some(toggle_ref) = toggle_ref.try_get() {
            toggle_ref.unchecked_into::<HtmlInputElement>().set_checked(completed);
        }
    }));

    let class = cloned!((todo, editing) => move || {
        format!("{} {}",
            if todo.get().completed { "completed" } else { "" },
            if *editing.get() { "editing" } else { "" }
        )
    });

    template! {
        li(class=class()) {
            div(class="view") {
                input(ref=toggle_ref, class="toggle", type="checkbox", on:input=toggle_completed)
                label(on:dblclick=handle_dblclick) {
                    (task())
                }
                button(class="destroy", on:click=handle_destroy)
            }

            (if *editing.get() {
                cloned!((todo, input_ref, handle_blur, handle_submit, handle_input) => template! {
                    input(ref=input_ref,
                        class="edit",
                        value=todo.get().task.clone(),
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
