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
    pub todos: Signal<Vec<Signal<Todo>>>,
}

impl AppState {
    fn add_todo(&self, task: String) {
        self.todos.set(
            self.todos
                .get()
                .as_ref()
                .clone()
                .into_iter()
                .chain(Some(Signal::new(Todo {
                    task,
                    completed: false,
                    id: Uuid::new_v4(),
                })))
                .collect(),
        )
    }

    fn remove_todo(&self, id: Uuid) {
        self.todos.set(
            self.todos
                .get()
                .iter()
                .filter(|todo| todo.get().id != id)
                .cloned()
                .collect(),
        );
    }

    fn toggle_completed(&self, id: Uuid) {
        for todo in self.todos.get().iter() {
            if todo.get().id == id {
                todo.set(Todo {
                    completed: !todo.get().completed,
                    ..todo.get().as_ref().clone()
                });
                break;
            }
        }
    }

    fn edit_todo_task(&self, id: Uuid, new_task: String) {
        for todo in self.todos.get().iter() {
            if todo.get().id == id {
                todo.set(Todo {
                    task: new_task,
                    ..todo.get().as_ref().clone()
                });
                break;
            }
        }
    }

    fn todos_left(&self) -> u32 {
        self.todos.get().iter().fold(
            0,
            |acc, todo| if todo.get().completed { acc } else { acc + 1 },
        )
    }

    fn toggle_complete_all(&self) {
        if self.todos_left() == 0 {
            // make all todos active
            for todo in self.todos.get().iter() {
                if todo.get().completed {
                    todo.set(Todo {
                        completed: false,
                        ..todo.get().as_ref().clone()
                    })
                }
            }
        } else {
            // make all todos completed
            for todo in self.todos.get().iter() {
                if !todo.get().completed {
                    todo.set(Todo {
                        completed: true,
                        ..todo.get().as_ref().clone()
                    })
                }
            }
        }
    }
}

fn App() -> TemplateResult {
    let app_state = AppState {
        todos: Signal::new(Vec::new()),
    };

    let todos_is_empty =
        create_selector(cloned!((app_state) => move || app_state.todos.get().len() == 0));

    template! {
        div(class="todomvc-wrapper") {
            section(class="todoapp") {
                header::Header(app_state.clone())

                (if !*todos_is_empty.get() {
                    log::info!("rendered");
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
