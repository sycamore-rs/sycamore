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
use sycamore::futures::{spawn_local, ScopeFuturesExt};
use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router};
const LATEST_MAJOR_VERSION: &str = "v0.7";
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
struct DarkMode(RcSignal<bool>);

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

fn switch<'a, G: Html>(ctx: ScopeRef<'a>, route: &'a ReadSignal<Routes>) -> View<G> {
    let cached_sidebar_data =
        ctx.create_ref(create_rc_signal(None::<(Option<String>, SidebarData)>));
    ctx.provide_context_ref(cached_sidebar_data);

    let fetch_docs_data = |url| {
        let data = ctx.create_resource(docs_preload(url));
        if cached_sidebar_data.get().is_none()
            || cached_sidebar_data.get().as_ref().as_ref().unwrap().0 != None
        {
            // Update sidebar
            let cached_sidebar_data = cached_sidebar_data.clone();
            spawn_local(async move {
                cached_sidebar_data.set(Some((None, get_sidebar(None).await)));
            });
        }
        data
    };
    let view = ctx.create_memo(on([route], move || match route.get().as_ref() {
        Routes::Index => view! { ctx,
            div(class="container mx-auto") {
                index::Index {}
            }
        },
        Routes::Docs(a, b) => {
            let data = fetch_docs_data(format!("/static/docs/{a}/{b}.json"));
            view! { ctx,
                (if let Some(data) = data.get().as_ref() {
                    if let Some(cached_sidebar_data) = cached_sidebar_data.get().as_ref() {
                        view! { ctx,
                            content::Content {
                                data: data.clone(),
                                sidebar: (
                                    "next".to_string(),
                                    cached_sidebar_data.1.clone(),
                                ),
                            }
                        }
                    } else {
                        view! { ctx, }
                    }
                } else {
                    view! { ctx, }
                })
            }
        }
        Routes::VersionedDocs(version, a, b) => {
            let version = version.clone();
            let data = fetch_docs_data(format!("/static/docs/{version}/{a}/{b}.json"));
            view! { ctx,
                (if let Some(data) = data.get().as_ref() {
                    if let Some(cached_sidebar_data) = cached_sidebar_data.get().as_ref() {
                        let version = version.clone();
                        view! { ctx,
                            content::Content {
                                data: data.clone(),
                                sidebar: (
                                    version,
                                    cached_sidebar_data.1.clone(),
                                ),
                            }
                        }
                    } else {
                        view! { ctx, }
                    }
                } else {
                    view! { ctx, }
                })
            }
        }
        Routes::NewsIndex => view! { ctx,
            news_index::NewsIndex {}
        },
        Routes::Post(name) => {
            let data = ctx.create_resource(docs_preload(format!("/static/posts/{}.json", name)));
            view! { ctx,
                (if let Some(data) = data.get().as_ref() {
                    view! { ctx,
                        content::Content {
                            data: data.clone(),
                        }
                    }
                } else {
                    view! { ctx, }
                })
            }
        }
        Routes::Versions => view! { ctx,
            versions::Versions {}
        },
        Routes::NotFound => view! { ctx,
            "404 Not Found"
        },
    }));

    view! { ctx,
        div(class="font-body pt-12 text-black dark:text-gray-200 bg-white dark:bg-gray-800 \
            min-h-screen transition-colors"
        ) {
            header::Header {}
            ((*view.get()).clone())
        }
    }
}

#[component]
fn App<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
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
    let dark_mode = DarkMode(create_rc_signal(dark_mode));
    ctx.provide_context(dark_mode);
    let DarkMode(dark_mode) = ctx.use_context::<DarkMode>();

    ctx.create_effect(move || {
        if let Some(local_storage) = &local_storage {
            local_storage
                .set_item("dark_mode", &dark_mode.get().to_string())
                .unwrap();
        }
    });

    view! { ctx,
        main(class=if *dark_mode.get() { "dark" } else { "" }) {
            (if *dark_mode.get() {
                view! { ctx, link(rel="stylesheet", href="/static/dark.css") }
            } else {
                view! { ctx, link(rel="stylesheet", href="/static/light.css") }
            })
            Router {
                integration: HistoryIntegration::new(),
                view: switch,
            }
        }
    }
}

fn main() {
    #[cfg(debug_assertions)]
    {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Debug).unwrap();
    }

    sycamore::render(|ctx| {
        view! { ctx, App {} }
    });
}
