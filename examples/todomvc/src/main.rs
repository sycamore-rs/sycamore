#![allow(non_snake_case)]

mod copyright;
mod footer;
mod header;
mod item;
mod list;

use maple_core::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    title: String,
    completed: bool,
    id: Uuid,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub todos: Signal<Vec<Signal<Todo>>>,
    pub filter: Signal<Filter>,
}

impl AppState {
    fn add_todo(&self, title: String) {
        self.todos.set(
            self.todos
                .get()
                .as_ref()
                .clone()
                .into_iter()
                .chain(Some(Signal::new(Todo {
                    title,
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

impl Default for AppState {
    fn default() -> Self {
        Self {
            todos: Signal::new(Vec::new()),
            filter: Signal::new(Filter::All),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    fn url(self) -> &'static str {
        match self {
            Filter::All => "./#",
            Filter::Active => "./#/active",
            Filter::Completed => "./#/completed",
        }
    }

    fn get_filter_from_hash() -> Self {
        let hash = web_sys::window().unwrap().location().hash().unwrap();

        match hash.as_str() {
            "#/active" => Filter::Active,
            "#/completed" => Filter::Completed,
            _ => Filter::All,
        }
    }
}

const KEY: &str = "todos-maple";

fn App() -> TemplateResult {
    let local_storage = web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .expect("user has not enabled localStorage");

    let todos = if let Ok(Some(app_state)) = local_storage.get_item(KEY) {
        serde_json::from_str(&app_state).unwrap_or_else(|_| Signal::new(Vec::new()))
    } else {
        Signal::new(Vec::new())
    };

    let app_state = AppState {
        todos,
        filter: Signal::new(Filter::get_filter_from_hash()),
    };

    create_effect(cloned!((local_storage, app_state) => move || {
        for todo in app_state.todos.get().iter() {
            todo.get(); // subscribe to changes in all todos
        }

        local_storage.set_item(KEY, &serde_json::to_string(app_state.todos.get().as_ref()).unwrap()).unwrap();
    }));

    let todos_is_empty =
        create_selector(cloned!((app_state) => move || app_state.todos.get().len() == 0));

    let todos_is_empty2 = todos_is_empty.clone();
    let app_state2 = app_state.clone();

    template! {
        div(class="todomvc-wrapper") {
            section(class="todoapp") {
                header::Header(app_state.clone())

                (if !*todos_is_empty.get() {
                    template! {
                        list::List(app_state.clone())
                    }
                } else {
                    TemplateResult::empty()
                })

                // FIXME: merge two if/else statements
                (if !*todos_is_empty2.get() {
                    template! {
                        footer::Footer(app_state2.clone())
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
