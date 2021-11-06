mod copyright;
mod filter;
mod footer;
mod header;
mod item;
mod list;

use log::Level::Debug;
use serde::{Deserialize, Serialize};
use sycamore::context::{ContextProvider, ContextProviderProps};
use sycamore::prelude::*;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
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

    fn todos_left(&self) -> usize {
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

    fn clear_completed(&self) {
        self.todos.set(
            self.todos
                .get()
                .iter()
                .filter(|todo| !todo.get().completed)
                .cloned()
                .collect(),
        );
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
            Filter::All => "#",
            Filter::Active => "#/active",
            Filter::Completed => "#/completed",
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

#[component(App<G>)]
fn app() -> View<G> {
    view! {
        div(class="todomvc-wrapper") {
            section(class="todoapp") {
                header::Header()
                list::List()
                footer::Footer()
            }
            copyright::Copyright()
        }
    }
}

const KEY: &str = "todos-sycamore";

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Debug).unwrap();

    // Initialize application state
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

    // Set up an effect that runs a function anytime app_state.todos changes
    create_effect(cloned!((local_storage, app_state) => move || {
        for todo in app_state.todos.get().iter() {
            todo.get(); // subscribe to changes in all todos
        }
        local_storage.set_item(KEY, &serde_json::to_string(app_state.todos.get().as_ref()).unwrap()).unwrap();
    }));

    /*
    The application's root component. We use a provider to 'provide' access
    to our app_state via the `use_context` API, which can be used from any
    level in the view tree.
    */
    sycamore::render(|| {
        view! {
            ContextProvider(ContextProviderProps {
                value: app_state,
                children: || view! {
                    App()
                }
            })
        }
    });
}
