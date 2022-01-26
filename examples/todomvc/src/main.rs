use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, KeyboardEvent};

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
        self.todos.set(
            self.todos
                .get()
                .as_ref()
                .clone()
                .into_iter()
                .chain(Some(create_rc_signal(Todo {
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

const KEY: &str = "todos-sycamore";

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|ctx| {
        view! { ctx,
            App()
        }
    });
}

#[component]
fn App<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    // Initialize application state
    let local_storage = web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .expect("user has not enabled localStorage");

    let todos = if let Ok(Some(app_state)) = local_storage.get_item(KEY) {
        serde_json::from_str(&app_state).unwrap_or_else(|_| create_rc_signal(Vec::new()))
    } else {
        create_rc_signal(Vec::new())
    };
    let app_state = AppState {
        todos,
        filter: create_rc_signal(Filter::get_filter_from_hash()),
    };
    ctx.provide_context(app_state);
    // Set up an effect that runs a function anytime app_state.todos changes
    ctx.create_effect(move || {
        let app_state = ctx.use_context::<AppState>();
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

    view! { ctx,
        div(class="todomvc-wrapper") {
            section(class="todoapp") {
                Header {}
                List {}
                Footer {}
            }
            Copyright {}
        }
    }
}

#[component]
pub fn Copyright<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    view! { ctx,
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
pub fn Header<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let app_state = ctx.use_context::<AppState>();
    let value = ctx.create_signal(String::new());
    let input_ref = ctx.create_node_ref();

    let handle_submit = |event: Event| {
        let event: KeyboardEvent = event.unchecked_into();

        if event.key() == "Enter" {
            let mut task = value.get().as_ref().clone();
            task = task.trim().to_string();

            if !task.is_empty() {
                app_state.add_todo(task);
                value.set("".to_string());
                input_ref
                    .get::<DomNode>()
                    .unchecked_into::<HtmlInputElement>()
                    .set_value("");
            }
        }
    };

    view! { ctx,
        header(class="header") {
            h1 { "todos" }
            input(ref=input_ref,
                class="new-todo",
                placeholder="What needs to be done?",
                // FIXME: find out why macro causes an error but manually expanding it doesn't.
                // bind:value=value
                on:keyup=handle_submit,
                on:change=|ev: Event| {
                    value.set(ev.target().unwrap().unchecked_into::<HtmlInputElement>().value())
                },
            )
        }
    }
}

#[component]
pub fn Item<G: Html>(ctx: ScopeRef, todo: RcSignal<Todo>) -> View<G> {
    let app_state = ctx.use_context::<AppState>();
    // Make `todo` live as long as the scope.
    let todo = ctx.create_ref(todo);

    let title = || todo.get().title.clone();
    let completed = ctx.create_selector(|| todo.get().completed);
    let id = todo.get().id;

    let editing = ctx.create_signal(false);
    let input_ref = ctx.create_node_ref();
    let value = ctx.create_signal("".to_string());

    let handle_input = |event: Event| {
        let target: HtmlInputElement = event.target().unwrap().unchecked_into();
        value.set(target.value());
    };

    let toggle_completed = |_| {
        todo.set(Todo {
            completed: !todo.get().completed,
            ..todo.get().as_ref().clone()
        });
    };

    let handle_dblclick = move |_| {
        editing.set(true);
        input_ref
            .get::<DomNode>()
            .unchecked_into::<HtmlInputElement>()
            .focus()
            .unwrap();
        value.set(title());
    };

    let handle_blur = move || {
        editing.set(false);

        let mut value = value.get().as_ref().clone();
        value = value.trim().to_string();

        if value.is_empty() {
            app_state.remove_todo(id);
        } else {
            todo.set(Todo {
                title: value,
                ..todo.get().as_ref().clone()
            })
        }
    };

    let handle_submit = move |event: Event| {
        let event: KeyboardEvent = event.unchecked_into();
        match event.key().as_str() {
            "Enter" => handle_blur(),
            "Escape" => {
                input_ref
                    .get::<DomNode>()
                    .unchecked_into::<HtmlInputElement>()
                    .set_value(&title());
                editing.set(false);
            }
            _ => {}
        }
    };

    let handle_destroy = move |_| {
        app_state.remove_todo(id);
    };

    // We need a separate signal for checked because clicking the checkbox will detach the binding
    // between the attribute and the view.
    let checked = ctx.create_signal(false);
    ctx.create_effect(|| {
        // Calling checked.set will also update the `checked` property on the input element.
        checked.set(*completed.get())
    });

    let class = || {
        format!(
            "{} {}",
            if *completed.get() { "completed" } else { "" },
            if *editing.get() { "editing" } else { "" }
        )
    };

    view! { ctx,
        li(class=class()) {
            div(class="view") {
                input(
                    class="toggle",
                    type="checkbox",
                    checked=*checked.get(),
                    on:input=toggle_completed,
                    // TODO: use `bind:value` syntax instead
                    /* bind:checked=checked, */
                )
                label(on:dblclick=handle_dblclick) {
                    (title())
                }
                button(class="destroy", on:click=handle_destroy)
            }

            (if *editing.get() {
                view! { ctx,
                    input(ref=input_ref,
                        class="edit",
                        value=todo.get().title.clone(),
                        on:blur=move |_| handle_blur(),
                        on:keyup=handle_submit,
                        on:input=handle_input,
                    )
                }
            } else {
                View::empty()
            })
        }
    }
}

#[component]
pub fn List<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let app_state = ctx.use_context::<AppState>();
    let todos_left = ctx.create_selector(|| app_state.todos_left());

    let filtered_todos = ctx.create_memo(|| {
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
    let checked = ctx.create_signal(false);
    ctx.create_effect(|| {
        // Calling checked.set will also update the `checked` property on the input element.
        checked.set(*todos_left.get() == 0)
    });

    view! { ctx,
        section(class="main") {
            input(
                id="toggle-all",
                class="toggle-all",
                type="checkbox",
                readonly=true,
                // TODO: fix this
                // bind:checked=checked,
                checked=*checked.get(),
                on:input=|_| app_state.toggle_complete_all()
            )
            label(for="toggle-all")

            ul(class="todo-list") {
                Keyed {
                    iterable: filtered_todos,
                    view: |ctx, todo| view! { ctx,
                        Item(todo)
                    },
                    key: |todo| todo.get().id,
                }
            }
        }
    }
}

#[component]
pub fn TodoFilter<G: Html>(ctx: ScopeRef, filter: Filter) -> View<G> {
    let app_state = ctx.use_context::<AppState>();
    let selected = move || filter == *app_state.filter.get();
    let set_filter = |filter| app_state.filter.set(filter);

    view! { ctx,
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
pub fn Footer<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let app_state = ctx.use_context::<AppState>();

    let items_text = || match app_state.todos_left() {
        1 => "item",
        _ => "items",
    };

    let has_completed_todos =
        ctx.create_selector(|| app_state.todos_left() < app_state.todos.get().len());

    let handle_clear_completed = |_| app_state.clear_completed();

    view! { ctx,
        footer(class="footer") {
            span(class="todo-count") {
                strong { (app_state.todos_left()) }
                span { " " (items_text()) " left" }
            }
            ul(class="filters") {
                TodoFilter(Filter::All)
                TodoFilter(Filter::Active)
                TodoFilter(Filter::Completed)
            }

            (if *has_completed_todos.get() {
                view! { ctx,
                    button(class="clear-completed", on:click=handle_clear_completed) {
                        "Clear completed"
                    }
                }
            } else {
                View::empty()
            })
        }
    }
}
