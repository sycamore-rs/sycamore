#![allow(non_snake_case)]

use maple_core::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

fn TodoItem(item: String) -> TemplateResult {
    let counter = Signal::new(0);

    let handle_click = cloned!((counter) => move |_| {
        log::info!("Clicked! New value = {}", *counter.get() + 1);
        counter.set(*counter.get() + 1);
    });

    template! {
        li{
            button(on:click=handle_click) {
                (counter.get()) " " (item.clone())
            }
        }
    }
}

fn App() -> TemplateResult {
    let todos = Signal::new(vec!["Hello".to_string(), "Hello again".to_string()]);
    let value = Signal::new(String::new());

    create_effect(cloned!((todos) => move || {
        log::info!("Todos changed: {:?}", todos.get());
    }));

    let handle_input = cloned!((value) => move |event: Event| {
        let target: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
        value.set(target.value());
    });

    let handle_add = cloned!((todos) => move |_| {
        let mut tmp = (*todos.get()).clone();
        tmp.push(value.get().as_ref().clone());
        todos.set(tmp);
    });

    let handle_remove = cloned!((todos) => move |_| {
        let mut tmp = (*todos.get()).clone();
        tmp.pop();
        todos.set(tmp);
    });

    let handle_remove_first = cloned!((todos) => move |_| {
        if !todos.get().is_empty() {
            todos.set(todos.get()[1..].into());
        }
    });

    template! {
        main {
            h1 {
                "todos"
            }

            input(placeholder="What needs to be done?", on:input=handle_input)
            button(on:click=handle_add) { "Add todo" }
            button(on:click=handle_remove) { "Remove last todo" }
            button(on:click=handle_remove_first) { "Remove first todo" }

            ul {
                Keyed(KeyedProps {
                    iterable: todos,
                    template: |item| {
                        template! {
                            TodoItem(item)
                        }
                    },
                    key: |item| item.clone()
                })
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
