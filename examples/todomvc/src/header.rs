use sycamore::{context::use_context, prelude::*};
use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent};

use crate::AppState;

#[component(Header<G>)]
pub fn header() -> View<G> {
    let app_state = use_context::<AppState>();
    let value = Signal::new(String::new());

    let handle_submit = cloned!((app_state, value) => move |event: Event| {
        let event: KeyboardEvent = event.unchecked_into();

        if event.key() == "Enter" {
            let mut task = value.get().as_ref().clone();
            task = task.trim().to_string();

            if !task.is_empty() {
                app_state.add_todo(task);
                value.set("".to_string());
            }
        }
    });

    view! {
        header(class="header") {
            h1 { "todos" }
            input(class="new-todo",
                placeholder="What needs to be done?",
                bind:value=value,
                on:keyup=handle_submit,
            )
        }
    }
}
