mod content;
mod header;
mod index;
mod sidebar;

use sycamore::prelude::*;
use sycamore_router::{BrowserRouter, Route};

#[derive(Debug, Route)]
enum Routes {
    #[to("/")]
    Index,
    #[to("/docs/<_>/<_>")]
    Docs(String, String),
    #[not_found]
    NotFound,
}

#[component(App<G>)]
fn app() -> Template<G> {
    template! {
        main {
            BrowserRouter(|route: Routes| {
                template! {
                    div(class="mt-12") {
                        header::Header()
                        (match &route {
                            Routes::Index => template! {
                                div(class="container mx-auto") {
                                    index::Index()
                                }
                            },
                            Routes::Docs(a, b) => template! {
                                content::Content(format!("/{}/{}", a, b))
                            },
                            Routes::NotFound => template! {
                                "404 Not Found"
                            },
                        })
                    }
                }
            })
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| template! { App() });
}
