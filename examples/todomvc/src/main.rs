#![allow(non_snake_case)]

mod copyright;
mod header;

use maple_core::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Todo {
    task: String,
    id: Uuid,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub todos: Signal<Vec<Todo>>,
}

impl AppState {
    fn add_todo(&self, task: String) {
        self.todos.set(
            self.todos
                .get()
                .as_ref()
                .clone()
                .into_iter()
                .chain(Some(Todo {
                    task,
                    id: Uuid::new_v4(),
                }))
                .collect(),
        )
    }
}

fn App() -> TemplateResult {
    let app_state = AppState {
        todos: Signal::new(Vec::new()),
    };

    create_effect(cloned!((app_state) => move || {
        log::info!("Todos changed: {:#?}", app_state.todos.get());
    }));

    template! {
        div(class="todomvc-wrapper") {
            section(class="todoapp") {
                header::Header(app_state)
            }

            copyright::Copyright()
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
