mod apis;
mod components;
mod pages;

use apis::types::StorySorting;
use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router};

#[derive(Debug, Route, Clone)]
enum AppRoutes {
    #[to("/")]
    Top,
    #[to("/new")]
    New,
    #[to("/best")]
    Best,
    #[to("/show")]
    Show,
    #[to("/user/<username>")]
    User { username: String },
    #[to("/item/<id>")]
    Item { id: i64 },
    #[not_found]
    NotFound,
}

#[component]
async fn Switch(route: Signal<AppRoutes>) -> View {
    view! {
        (match route.get_clone() {
            AppRoutes::Top => view! { pages::stories::Stories(sorting=StorySorting::Top) },
            AppRoutes::New => view! { pages::stories::Stories(sorting=StorySorting::New) },
            AppRoutes::Best => view! { pages::stories::Stories(sorting=StorySorting::Best) },
            AppRoutes::Show => view! { pages::stories::Stories(sorting=StorySorting::Show) },
            AppRoutes::User { username } => view! { pages::user::User(username=username.clone()) },
            AppRoutes::Item { id } => view! { pages::item::Item(id=id) },
            AppRoutes::NotFound => view! { "404 Page Not Found"}
        })
    }
}

#[component]
fn App() -> View {
    view! {
        Router(
            integration=HistoryIntegration::new(),
            view=|route: Signal<AppRoutes>| view! {
                div(class="app mb-2") {
                    components::header::Header()
                    div(class="container mx-auto") {
                        Switch(route=route)
                    }
                }
            }
        )
    }
}

fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    console_log::init().expect("error initializing logger");

    sycamore::render(App);
}
