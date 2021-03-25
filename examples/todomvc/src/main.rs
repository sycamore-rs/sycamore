#![allow(non_snake_case)]

mod copyright;
mod header;
mod item;
mod list;

use maple_core::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    task: String,
    completed: bool,
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
                    completed: false,
                    id: Uuid::new_v4(),
                }))
                .collect(),
        )
    }

    fn remove_todo(&self, id: Uuid) {
        self.todos.set(
            self.todos
                .get()
                .iter()
                .filter(|todo| todo.id != id)
                .cloned()
                .collect(),
        );
    }

    fn toggle_completed(&self, id: Uuid) {
        self.todos.set(
            self.todos
                .get()
                .iter()
                .map(|todo| {
                    if todo.id == id {
                        Todo {
                            completed: !todo.completed,
                            ..todo.clone()
                        }
                    } else {
                        todo.clone()
                    }
                })
                .collect(),
        );
    }

    fn edit_todo_task(&self, id: Uuid, new_task: String) {
        self.todos.set(
            self.todos
                .get()
                .iter()
                .map(|todo| {
                    if todo.id == id {
                        Todo {
                            task: new_task.clone(),
                            ..todo.clone()
                        }
                    } else {
                        todo.clone()
                    }
                })
                .collect(),
        );
    }
}

fn App() -> TemplateResult {
    let app_state = AppState {
        todos: Signal::new(Vec::new()),
    };

    let todos_is_empty =
        create_selector(cloned!((app_state) => move || app_state.todos.get().len() > 0));

    template! {
        div(class="todomvc-wrapper") {
            section(class="todoapp") {
                header::Header(app_state.clone())

                (if *todos_is_empty.get() {
                    template! {
                        list::List(app_state.clone())
                    }
                } else {
                    TemplateResult::empty()
                })
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
