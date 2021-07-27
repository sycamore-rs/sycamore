mod content;
mod header;
mod index;
mod news_index;
mod sidebar;
mod versions;

use content::MarkdownPage;
use reqwasm::http::Request;
use serde_lite::Deserialize;
use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router, RouterProps};

const LATEST_MAJOR_VERSION: &str = "v0.5";
const NEXT_VERSION: &str = "next";

async fn docs_preload(path: Vec<String>) -> MarkdownPage {
    let pathname = format!("/static/docs/{}.json", path.join("/"));

    let text = Request::get(&pathname).send().await.unwrap().text().await;
    if let Ok(text) = text {
        let intermediate = serde_json::from_str(&text).unwrap();
        MarkdownPage::deserialize(&intermediate).unwrap()
    } else {
        todo!("error handling");
    }
}

#[derive(Debug, Route)]
enum Routes {
    #[to("/")]
    Index,
    #[to("/docs/<_>/<_>")]
    #[preload(|path: Vec<String>| docs_preload(path[1..].to_vec()))]
    Docs(String, String, MarkdownPage),
    #[to("/docs/<_>/<_>/<_>")]
    #[preload(|path: Vec<String>| docs_preload(path[1..].to_vec()))]
    VersionedDocs(String, String, String, MarkdownPage),
    #[to("/news")]
    NewsIndex,
    #[to("/news/<_>")]
    #[preload(|path: Vec<String>| docs_preload(path[1..].to_vec()))]
    Post(String, MarkdownPage),
    #[to("/versions")]
    Versions,
    #[not_found]
    NotFound,
}

fn switch<G: GenericNode>(route: Routes) -> Template<G> {
    let template = match route {
        Routes::Index => template! {
            div(class="container mx-auto") {
                index::Index()
            }
        },
        Routes::Docs(a, b, data) => {
            template! {
                content::Content(content::ContentProps {
                    pathname: format!("/static/docs/{}/{}.json", a, b),
                    data,
                    sidebar_version: Some("next".to_string()),
                })
            }
        }
        Routes::VersionedDocs(version, a, b, data) => template! {
            content::Content(content::ContentProps {
                pathname: format!("/static/docs/{}/{}/{}.json", version, a, b),
                data,
                sidebar_version: Some(version.clone()),
            })
        },
        Routes::NewsIndex => template! {
            news_index::NewsIndex()
        },
        Routes::Post(post, data) => template! {
            content::Content(content::ContentProps {
                pathname: format!("/static/posts/{}.json", post),
                data,
                sidebar_version: None,
            })
        },
        Routes::Versions => template! {
            versions::Versions()
        },
        Routes::NotFound => template! {
            "404 Not Found"
        },
    };

    template! {
        div(class="mt-12") {
            header::Header()
            (template)
        }
    }
}

#[component(App<G>)]
fn app() -> Template<G> {
    template! {
        main {
            Router(RouterProps::new(HistoryIntegration::new(), switch))
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| template! { App() });
}
