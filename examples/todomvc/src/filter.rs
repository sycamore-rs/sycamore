use sycamore::context::use_context;
use sycamore::prelude::*;

use crate::{AppState, Filter};

#[component(TodoFilter<G>)]
pub fn todo_filter(filter: Filter) -> View<G> {
    let app_state = use_context::<AppState>();
    let selected = cloned!((app_state) => move || filter == *app_state.filter.get());
    let set_filter = cloned!((app_state) => move |filter| {
        app_state.filter.set(filter)
    });

    view! {
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
}
