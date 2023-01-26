use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use uuid::Uuid;
use web_sys::{HtmlInputElement, KeyboardEvent};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Todo {
    title: String,
    completed: bool,
    id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Default for Filter {
    fn default() -> Self {
        Self::All
    }
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

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub todos: RcSignal<Vec<RcSignal<Todo>>>,
    pub filter: RcSignal<Filter>,
}

impl AppState {
    fn add_todo(&self, title: String) {
        self.todos.modify().push(create_rc_signal(Todo {
            title,
            completed: false,
            id: Uuid::new_v4(),
        }))
    }

    fn remove_todo(&self, id: Uuid) {
        self.todos.modify().retain(|todo| todo.get().id != id);
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
        self.todos.modify().retain(|todo| !todo.get().completed);
    }
}

const KEY: &str = "todos-sycamore";

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(App);
}

#[component]
fn App(cx: Scope) -> View {
    // Initialize application state from localStorage.
    let local_storage = web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .expect("user has not enabled localStorage");

    let todos = if let Ok(Some(app_state)) = local_storage.get_item(KEY) {
        serde_json::from_str(&app_state).unwrap_or_default()
    } else {
        Default::default()
    };
    let app_state = AppState {
        todos,
        filter: create_rc_signal(Filter::get_filter_from_hash()),
    };
    let app_state = provide_context(cx, app_state);

    // Set up an effect that runs whenever app_state.todos changes to save the todos to
    // localStorage.
    create_effect(cx, move || {
        for todo in app_state.todos.get().iter() {
            todo.track();
        }
        local_storage
            .set_item(
                KEY,
                &serde_json::to_string(app_state.todos.get().as_ref()).unwrap(),
            )
            .unwrap();
    });

    let todos_empty = create_selector(cx, || app_state.todos.get().is_empty());

    view! { cx,
        div(class="todomvc-wrapper") {
            section(class="todoapp") {
                Header {}
                ((!*todos_empty.get()).then(|| view! { cx,
                    List {}
                    Footer {}
                }))
            }
            Copyright {}
        }
    }
}

#[component]
pub fn Copyright(cx: Scope) -> View {
    view! { cx,
        footer(class="info") {
            p { "Double click to edit a todo" }
            p {
                "Created by "
                a(href="https://github.com/lukechu10", target="_blank") { "lukechu10" }
            }
            p {
                "Part of "
                a(href="http://todomvc.com") { "TodoMVC" }
            }
        }
    }
}

#[component]
pub fn Header(cx: Scope) -> View {
    let app_state = use_context::<AppState>(cx);
    let input_value = create_signal(cx, String::new());

    let handle_keyup = |event: KeyboardEvent| {
        if event.key() == "Enter" {
            let mut task = input_value.get().as_ref().clone();
            task = task.trim().to_string();

            if !task.is_empty() {
                app_state.add_todo(task);
                // Reset input field.
                input_value.set("".to_string());
            }
        }
    };

    view! { cx,
        header(class="header") {
            h1 { "todos" }
            input(class="new-todo",
                placeholder="What needs to be done?",
                bind:value=input_value,
                on:keyup=handle_keyup,
            )
        }
    }
}

#[component(inline_props)]
pub fn Item(cx: Scope, todo: RcSignal<Todo>) -> View {
    let app_state = use_context::<AppState>(cx);
    // Make `todo` live as long as the scope.
    let todo = create_ref(cx, todo);

    let title = || todo.get().title.clone();
    let completed = create_selector(cx, || todo.get().completed);
    let id = todo.get().id;

    let is_editing = create_signal(cx, false);
    let input_ref: &NodeRef<WebNode> = create_node_ref(cx);
    let input_value = create_signal(cx, "".to_string());

    let toggle_completed = |_| todo.modify().completed = !todo.get().completed;

    let handle_dblclick = move |_| {
        is_editing.set(true);
        input_ref
            .get()
            .unchecked_into::<HtmlInputElement>()
            .focus()
            .unwrap();
        input_value.set(title());
    };

    let handle_blur = move || {
        is_editing.set(false);

        let mut value = input_value.get().as_ref().clone();
        value = value.trim().to_string();

        if value.is_empty() {
            app_state.remove_todo(id);
        } else {
            todo.modify().title = value;
        }
    };

    let handle_keyup = move |event: KeyboardEvent| match event.key().as_str() {
        "Enter" => handle_blur(),
        "Escape" => is_editing.set(false),
        _ => {}
    };

    let handle_destroy = move |_| {
        app_state.remove_todo(id);
    };

    // We need a separate signal for checked because clicking the checkbox will detach the binding
    // between the attribute and the view.
    let checked = create_signal(cx, false);
    create_effect(cx, || {
        // Calling checked.set will also update the `checked` property on the input element.
        checked.set(*completed.get())
    });

    let class = || {
        format!(
            "{} {}",
            if *completed.get() { "completed" } else { "" },
            if *is_editing.get() { "editing" } else { "" }
        )
    };

    view! { cx,
        li(class=class()) {
            div(class="view") {
                input(
                    class="toggle",
                    _type="checkbox",
                    on:input=toggle_completed,
                    bind:checked=checked
                )
                label(on:dblclick=handle_dblclick) {
                    (title())
                }
                button(class="destroy", on:click=handle_destroy)
            }

            (is_editing.get().then(|| view! { cx,
                input(_ref=input_ref,
                    class="edit",
                    bind:value=input_value,
                    on:blur=move |_| handle_blur(),
                    on:keyup=handle_keyup,
                )
            }))
        }
    }
}

#[component]
pub fn List(cx: Scope) -> View {
    let app_state = use_context::<AppState>(cx);
    let todos_left = create_selector(cx, || app_state.todos_left());

    let filtered_todos = create_memo(cx, || {
        app_state
            .todos
            .get()
            .iter()
            .filter(|todo| match *app_state.filter.get() {
                Filter::All => true,
                Filter::Active => !todo.get().completed,
                Filter::Completed => todo.get().completed,
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    // We need a separate signal for checked because clicking the checkbox will detach the binding
    // between the attribute and the view.
    let checked = create_signal(cx, false);
    create_effect(cx, || {
        // Calling checked.set will also update the `checked` property on the input element.
        checked.set(*todos_left.get() == 0)
    });

    view! { cx,
        section(class="main") {
            input(
                id="toggle-all",
                class="toggle-all",
                _type="checkbox",
                readonly=true,
                bind:checked=checked,
                on:input=|_| app_state.toggle_complete_all()
            )
            label(_for="toggle-all")

            ul(class="todo-list") {
                Keyed(
                    iterable=filtered_todos,
                    view=|cx, todo| view! { cx,
                        Item(todo=todo)
                    },
                    key=|todo| todo.get().id,
                )
            }
        }
    }
}

#[component(inline_props)]
pub fn TodoFilter(cx: Scope, filter: Filter) -> View {
    let app_state = use_context::<AppState>(cx);
    let selected = move || filter == *app_state.filter.get();
    let set_filter = |filter| app_state.filter.set(filter);

    view! { cx,
        li {
            a(
                class=if selected() { "selected" } else { "" },
                href=filter.url(),
                on:click=move |_| set_filter(filter),
            ) {
                (format!("{filter:?}"))
            }
        }
    }
}

#[component]
pub fn Footer(cx: Scope) -> View {
    let app_state = use_context::<AppState>(cx);

    let items_text = || match app_state.todos_left() {
        1 => "item",
        _ => "items",
    };

    let has_completed_todos =
        create_selector(cx, || app_state.todos_left() < app_state.todos.get().len());

    let handle_clear_completed = |_| app_state.clear_completed();

    view! { cx,
        footer(class="footer") {
            span(class="todo-count") {
                strong { (app_state.todos_left()) }
                span { " " (items_text()) " left" }
            }
            ul(class="filters") {
                TodoFilter(filter=Filter::All)
                TodoFilter(filter=Filter::Active)
                TodoFilter(filter=Filter::Completed)
            }

            (has_completed_todos.get().then(|| view! { cx,
                button(class="clear-completed", on:click=handle_clear_completed) {
                    "Clear completed"
                }
            }))
        }
    }
}
