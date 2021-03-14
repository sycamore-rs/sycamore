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
    let todos: Signal<Vec<String>> = Signal::new(Vec::new());

    let value = Signal::new(String::new());

    let handle_input = cloned!((value) => move |event: Event| {
        let target: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
        value.set(target.value());
    });

    let handle_click = cloned!((todos) => move |_| {
        let mut tmp = todos.get().as_ref().clone();
        tmp.push(value.get().as_ref().clone());

        todos.set(tmp);
    });

    template! {
        main {
            h1 {
                "todos"
            }

            input(placeholder="What needs to be done?", on:input=handle_input)
            button(on:click=handle_click) { "Add todo" }

            ul {
                h1 { "Test" }
                (todos.get().iter().map(|todo| template! { TodoItem(todo.clone()) }).collect::<TemplateList>())
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
