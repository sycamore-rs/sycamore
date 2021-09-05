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
use wasm_bindgen_futures::spawn_local;

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

fn switch<G: GenericNode>(route: StateHandle<Routes>) -> Template<G> {
    let template = Signal::new(Template::empty());
    create_effect(cloned!((template) => move || {
        let route = route.get();
        spawn_local(cloned!((template) => async move {
            let t = match route.as_ref() {
                Routes::Index => template! {
                    div(class="container mx-auto") {
                        index::Index()
                    }
                },
                Routes::Docs(a, b) => {
                    let data = docs_preload(format!("/static/docs/{}/{}.json", a, b)).await;
                    template! {
                        content::Content(content::ContentProps {
                            data,
                            sidebar_version: Some("next".to_string()),
                        })
                    }
                }
                Routes::VersionedDocs(version, a, b) => {
                    let data = docs_preload(format!("/static/docs/{}/{}/{}.json", version, a, b)).await;
                    template! {
                        content::Content(content::ContentProps {
                            data,
                            sidebar_version: Some(version.clone()),
                        })
                    }
                }
                Routes::NewsIndex => template! {
                    news_index::NewsIndex()
                },
                Routes::Post(name) => {
                    let data = docs_preload(format!("/static/posts/{}.json", name)).await;
                    template! {
                        content::Content(content::ContentProps {
                            data,
                            sidebar_version: None,
                        })
                    }
                }
                Routes::Versions => template! {
                    versions::Versions()
                },
                Routes::NotFound => template! {
                    "404 Not Found"
                },
            };
            template.set(t);
        }));
    }));

    template! {
        div(class="mt-12") {
            header::Header()
            (template.get().as_ref().clone())
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
