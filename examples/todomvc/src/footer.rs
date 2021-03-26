use maple_core::prelude::*;

use crate::{AppState, Filter};

pub fn Footer(app_state: AppState) -> TemplateResult {
    let items_text = cloned!((app_state) => move || {
        match app_state.todos_left() {
            1 => "item",
            _ => "items"
        }
    });

    let app_state2 = app_state.clone();

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
        }
    }
}
