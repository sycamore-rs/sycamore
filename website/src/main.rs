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

async fn docs_preload(path: String) -> MarkdownPage {
    let text = Request::get(&path).send().await.unwrap().text().await;
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
    #[preload(|path: Vec<String>| docs_preload(format!("/static/docs/{}.json", path[1..].join("/"))))]
    Docs(String, String, MarkdownPage),
    #[to("/docs/<_>/<_>/<_>")]
    #[preload(|path: Vec<String>| docs_preload(format!("/static/docs/{}.json", path[1..].join("/"))))]
    VersionedDocs(String, String, String, MarkdownPage),
    #[to("/news")]
    NewsIndex,
    #[to("/news/<_>")]
    #[preload(|path: Vec<String>| docs_preload(format!("/static/posts/{}.json", path[1..].join("/"))))]
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
        Routes::Docs(_, _, data) => {
            template! {
                content::Content(content::ContentProps {
                    data,
                    sidebar_version: Some("next".to_string()),
                })
            }
        }
        Routes::VersionedDocs(version, _, _, data) => template! {
            content::Content(content::ContentProps {
                data,
                sidebar_version: Some(version.clone()),
            })
        },
        Routes::NewsIndex => template! {
            news_index::NewsIndex()
        },
        Routes::Post(_, data) => template! {
            content::Content(content::ContentProps {
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
