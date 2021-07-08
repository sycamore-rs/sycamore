mod content;
mod header;
mod index;
mod news_index;
mod sidebar;

use sycamore::prelude::*;
use sycamore_router::{BrowserRouter, Route};

#[derive(Debug, Route)]
enum Routes {
    #[to("/")]
    Index,
    #[to("/docs/<_>/<_>")]
    Docs(String, String),
    #[to("/news")]
    NewsIndex,
    #[to("/news/<_>")]
    Post(String),
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
                                content::Content(content::ContentProps {
                                    pathname: format!("/markdown/{}/{}.md", a, b),
                                    show_sidebar: true,
                                })
                            },
                            Routes::NewsIndex => template! {
                                news_index::NewsIndex()
                            },
                            Routes::Post(post) => template! {
                                content::Content(content::ContentProps {
                                    pathname: format!("/posts/{}.md", post),
                                    show_sidebar: false,
                                })
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
