mod content;
mod header;
mod index;
mod news_index;
mod sidebar;
mod versions;

use sycamore::prelude::*;
use sycamore_router::{BrowserRouter, Route};

const LATEST_MAJOR_VERSION: &str = "v0.5";
const NEXT_VERSION: &str = "next";

#[derive(Debug, Route)]
enum Routes {
    #[to("/")]
    Index,
    #[to("/docs/<_>/<_>")]
    Docs(String, String),
    #[to("/docs/<_>/<_>/<_>")]
    VersionedDocs(String, String, String),
    #[to("/news")]
    NewsIndex,
    #[to("/news/<_>")]
    Post(String),
    #[to("/versions")]
    Versions,
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
                                    pathname: format!("/static/docs/{}/{}.json", a, b),
                                    sidebar_version: Some("next".to_string()),
                                })
                            },
                            Routes::VersionedDocs(version, a, b) => template! {
                                content::Content(content::ContentProps {
                                    pathname: format!("/static/docs/{}/{}/{}.json", version, a, b),
                                    sidebar_version: Some(version.clone()),
                                })
                            },
                            Routes::NewsIndex => template! {
                                news_index::NewsIndex()
                            },
                            Routes::Post(post) => template! {
                                content::Content(content::ContentProps {
                                    pathname: format!("/static/posts/{}.json", post),
                                    sidebar_version: None,
                                })
                            },
                            Routes::Versions => template! {
                                versions::Versions()
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
