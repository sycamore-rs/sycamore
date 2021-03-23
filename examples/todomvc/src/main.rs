#![allow(non_snake_case)]

use maple_core::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

fn TodoItem(item: String) -> TemplateResult {
    template! {
        li { (item.clone()) }
    }
}

fn App() -> TemplateResult {
    let todos = SignalVec::new();
    let todos_template = todos.map(|todo: &String| template! { TodoItem(todo.clone()) });

    let value = Signal::new(String::new());

    let handle_input = cloned!((value) => move |event: Event| {
        let target: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
        value.set(target.value());
    });

    let handle_add = cloned!((todos) => move |_| {
        todos.insert(0, value.get().as_ref().clone());
    });

    let handle_remove = cloned!((todos) => move |_| {
        todos.pop();
    });

    template! {
        main {
            h1 {
                "todos"
            }

            input(placeholder="What needs to be done?", on:input=handle_input)
            button(on:click=handle_add) { "Add todo" }
            button(on:click=handle_remove) { "Remove last todo" }

            ul {
                (todos_template.template_list())
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
