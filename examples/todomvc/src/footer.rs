use sycamore::context::use_context;
use sycamore::prelude::*;

use crate::{
    AppState,
    Filter,
    filter::TodoFilter,
};

#[component(Footer<G>)]
pub fn footer() -> Template<G> {
    let app_state = use_context::<AppState>();

    let items_text = cloned!((app_state) => move || {
        match app_state.todos_left() {
            1 => "item",
            _ => "items"
        }
    });

    let has_completed_todos = create_selector(cloned!((app_state) => move || {
        app_state.todos_left() < app_state.todos.get().len()
    }));

    let handle_clear_completed = cloned!((app_state) => move |_| {
        app_state.clear_completed()
    });

    template! {
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
                cloned!((handle_clear_completed) => template! {
                    button(class="clear-completed", on:click=handle_clear_completed) {
                        "Clear completed"
                    }
                })
            } else {
                Template::empty()
            })
        }
    }
}
