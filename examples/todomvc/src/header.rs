use maple_core::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, KeyboardEvent};

use crate::AppState;

pub fn Header(app_state: AppState) -> TemplateResult {
    let value = Signal::new(String::new());

    let input_ref = NodeRef::new();

    let handle_input = cloned!((value) => move |event: Event| {
        let target: HtmlInputElement = event.target().unwrap().unchecked_into();
        value.set(target.value());
    });

    let handle_submit = cloned!((value, input_ref) => move |event: Event| {
        let event: KeyboardEvent = event.unchecked_into();

        if event.key() == "Enter" {
            let mut task = value.get().as_ref().clone();
            task = task.trim().to_string();

            if !task.is_empty() {
                app_state.add_todo(task);
                value.set("".to_string());
                input_ref.get().unchecked_into::<HtmlInputElement>().set_value(""); // FIXME: bind to value property instead of attribute
            }
        }
    });

    template! {
        header(class="header") {
            h1 { "todos" }
            input(ref=input_ref,
                class="new-todo",
                placeholder="What needs to be done?",
                value=value.get(),
                on:input=handle_input,
                on:keyup=handle_submit,
            )
        }
    }
}
