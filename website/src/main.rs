mod content;
mod header;
mod index;
mod news_index;
mod sidebar;
mod versions;

use content::MarkdownPage;
use reqwasm::http::Request;
use serde_lite::Deserialize;
use sidebar::SidebarData;
use sycamore::context::{use_context, ContextProvider, ContextProviderProps};
use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router, RouterProps};
use wasm_bindgen_futures::spawn_local;

const LATEST_MAJOR_VERSION: &str = "v0.6";
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

#[derive(Clone)]
struct DarkMode(Signal<bool>);

async fn docs_preload(path: String) -> MarkdownPage {
    let text = Request::get(&path).send().await.unwrap().text().await;
    if let Ok(text) = text {
        let intermediate = serde_json::from_str(&text).unwrap();
        MarkdownPage::deserialize(&intermediate).unwrap()
    } else {
        todo!("error handling");
    }
}

async fn get_sidebar(version: Option<&str>) -> SidebarData {
    let url = if let Some(version) = version {
        format!("/static/docs/{}/sidebar.json", version)
    } else {
        "/static/docs/sidebar.json".to_string()
    };
    let text = Request::get(&url).send().await.unwrap().text().await;
    if let Ok(text) = text {
        let intermediate = serde_json::from_str(&text).unwrap();
        SidebarData::deserialize(&intermediate).unwrap()
    } else {
        todo!("error handling");
    }
}

fn switch<G: GenericNode>(route: StateHandle<Routes>) -> Template<G> {
    let template = Signal::new(Template::empty());
    let cached_sidebar_data: Signal<Option<(Option<String>, SidebarData)>> = Signal::new(None);
    create_effect(cloned!((template) => move || {
        let route = route.get();
        spawn_local(cloned!((template, cached_sidebar_data) => async move {
            let t = match route.as_ref() {
                Routes::Index => template! {
                    div(class="container mx-auto") {
                        index::Index()
                    }
                },
                Routes::Docs(a, b) => {
                    let data = docs_preload(format!("/static/docs/{}/{}.json", a, b)).await;
                    if cached_sidebar_data.get().is_none()
                        || cached_sidebar_data.get().as_ref().as_ref().unwrap().0 != None {
                        // Update sidebar
                        cached_sidebar_data.set(Some((None, get_sidebar(None).await)));
                    }
                    template! {
                        content::Content(content::ContentProps {
                            data,
                            sidebar: Some((
                                "next".to_string(),
                                cached_sidebar_data.get().as_ref().as_ref().unwrap().1.clone(),
                            )),
                        })
                    }
                }
                Routes::VersionedDocs(version, a, b) => {
                    let data = docs_preload(format!("/static/docs/{}/{}/{}.json", version, a, b)).await;
                    if cached_sidebar_data.get().is_none()
                        || cached_sidebar_data.get().as_ref().as_ref().unwrap().0.as_deref() != Some(version) {
                        // Update sidebar
                        cached_sidebar_data.set(Some((Some(version.clone()), get_sidebar(Some(version)).await)));
                    }
                    template! {
                        content::Content(content::ContentProps {
                            data,
                            sidebar: Some((
                                version.clone(),
                                cached_sidebar_data.get().as_ref().as_ref().unwrap().1.clone(),
                            )),
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
                            sidebar: None,
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
        div(class="pt-12 text-black dark:text-gray-200 bg-white dark:bg-gray-800 \
            min-h-screen transition-colors") {
            header::Header()
            (template.get().as_ref())
        }
    }
}

#[component(App<G>)]
fn app() -> Template<G> {
    let DarkMode(dark_mode) = use_context::<DarkMode>();
    let dark_mode2 = dark_mode.clone();

    template! {
        main(class=if *dark_mode2.get() { "dark" } else { "" }) {
            (if *dark_mode.get() {
                template! { link(rel="stylesheet", href="/static/dark.css") }
            } else {
                template! { link(rel="stylesheet", href="/static/light.css") }
            })
            Router(RouterProps::new(HistoryIntegration::new(), switch))
        }
    }
}

fn main() {
    #[cfg(debug_assertions)]
    {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Debug).unwrap();
    }

    let local_storage = web_sys::window().unwrap().local_storage().unwrap();
    // Get dark mode from media query.
    let dark_mode_mq = web_sys::window()
        .unwrap()
        .match_media("(prefers-color-scheme: dark)")
        .unwrap()
        .unwrap()
        .matches();
    let dark_mode = if let Some(local_storage) = &local_storage {
        let dark_mode_ls = local_storage.get_item("dark_mode").unwrap();
        dark_mode_ls.as_deref() == Some("true") || (dark_mode_ls.is_none() && dark_mode_mq)
    } else {
        dark_mode_mq
    };
    let dark_mode = DarkMode(Signal::new(dark_mode));

    create_effect(cloned!((dark_mode) => move || {
        if let Some(local_storage) = &local_storage {
            local_storage.set_item("dark_mode", &dark_mode.0.get().to_string()).unwrap();
        }
    }));

    sycamore::render(|| {
        template! {
            ContextProvider(ContextProviderProps {
                value: dark_mode,
                children: || template! {
                    App()
                },
            })
        }
    });
}
