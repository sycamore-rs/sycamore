use sycamore::prelude::*;

use crate::{AppState, Filter};

#[component(Footer<G>)]
pub fn footer(app_state: AppState) -> TemplateResult<G> {
    let items_text = cloned!((app_state) => move || {
        match app_state.todos_left() {
            1 => "item",
            _ => "items"
        }
    });

    let has_completed_todos = create_selector(cloned!((app_state) => move || {
        app_state.todos_left() < app_state.todos.get().len()
    }));

    let app_state2 = app_state.clone();
    let app_state3 = app_state.clone();

    template! {
        footer(class="footer") {
            span(class="todo-count") {
                strong { (app_state.todos_left()) }
                span { " " (items_text()) " left" }
            }
            ul(class="filters") {
                Indexed(IndexedProps {
                    iterable: Signal::new(vec![Filter::All, Filter::Active, Filter::Completed]).handle(),
                    template: cloned!((app_state2) => move |filter| {
                        let selected = cloned!((app_state2) => move || filter == *app_state2.filter.get());
                        let set_filter = cloned!((app_state2) => move |filter| {
                            app_state2.filter.set(filter)
                        });

                        template! {
                            li {
                                a(
                                    class=if selected() { "selected" } else { "" },
                                    href=filter.url(),
                                    on:click=move |_| set_filter(filter),
                                ) {
                                    (format!("{:?}", filter))
                                }
                            }
                        }
                    })
                })
            }

            (if *has_completed_todos.get() {
                template! {
                    button(class="clear-completed", on:click=cloned!((app_state3) => move |_| app_state3.clear_completed())) {
                        "Clear completed"
                    }
                }
            } else {
                TemplateResult::empty()
            })
        }
    }
}
