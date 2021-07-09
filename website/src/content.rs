use reqwasm::http::Request;
use serde_lite::Deserialize;
use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

// Sync definition with docs/build.rs
#[derive(Deserialize)]
struct MarkdownPage {
    html: String,
    outline: Vec<Outline>,
}

// Sync definition with docs/build.rs
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Outline {
    name: String,
    children: Vec<Outline>,
}

#[wasm_bindgen(inline_js = "export function highlight_all() { hljs.highlightAll(); }")]
extern "C" {
    fn highlight_all();
}

#[component(OutlineView<G>)]
pub fn outline_view(outline: StateHandle<Vec<Outline>>) -> Template<G> {
    template! {
        ul(class="mt-4 text-sm pl-2 border-l border-gray-400") {
            Indexed(IndexedProps {
                iterable: outline,
                template: |item| {
                    let Outline { name, children } = item;
                    let nested = children.iter().map(|x| {
                        let name = x.name.clone();
                        let href = format!("#{}", x.name.trim().to_lowercase().replace(" ", "-"));
                        template! {
                            li {
                                a(href=href) {
                                    (name)
                                }
                            }
                        }
                    }).collect();
                    let nested = Template::new_fragment(nested);

                    let href = format!("#{}", name.trim().to_lowercase().replace(" ", "-"));

                    template! {
                        li {
                            a(href=href) {
                                (name)
                            }
                            ul(class="ml-3") {
                                (nested)
                            }
                        }
                    }
                }
            })
        }
    }
}

pub struct ContentProps {
    pub pathname: String,
    pub sidebar_version: Option<String>,
}

#[component(Content<G>)]
pub fn content(
    ContentProps {
        pathname,
        sidebar_version,
    }: ContentProps,
) -> Template<G> {
    let show_sidebar = sidebar_version.is_some();

    let docs_container_ref = NodeRef::<G>::new();

    let html = Signal::new(None::<String>);
    let outline = Signal::new(Vec::new());

    create_effect(cloned!((html, docs_container_ref) => move || {
        if let Some(html) = html.get().as_ref() {
            docs_container_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlElement>()
                .set_inner_html(&html);
            highlight_all();
        }
    }));

    wasm_bindgen_futures::spawn_local(cloned!((html, outline) => async move {
        let text = Request::get(&pathname).send().await.unwrap().text().await;
        if let Ok(text) = text{
            let intermediate = serde_json::from_str(&text).unwrap();
            let markdown_page = MarkdownPage::deserialize(&intermediate).unwrap();
            html.set(Some(markdown_page.html));
            outline.set(markdown_page.outline);
        } else {
            // TODO: error handling
        }
    }));

    let sidebar_version0 = sidebar_version.clone();

    template! {
        div(class="flex w-full") {
            (if show_sidebar {
                template! {
                    div(class="flex-none") {
                        crate::sidebar::Sidebar(sidebar_version.clone().unwrap())
                    }
                }
            } else {
                template! {}
            })
            div(class="flex-1 container mx-auto") {
                div(
                    class=format!("content min-w-0 pr-4 mb-2 lg:mr-44 {}",
                    if show_sidebar { "" } else { "container mx-auto lg:ml-auto lg:mr-44" }),
                ) {
                    (if sidebar_version0.as_deref() == Some(crate::NEXT_VERSION) {
                        template! {
                            div(class="bg-yellow-500 text-white w-full rounded-md mt-4 px-4 py-3") {
                                p { "This is unreleased documentation for Sycamore next version." }
                                p {
                                    "For up-to-date documentation, see the "
                                    a(href=format!("/docs/{}/getting_started/hello_world", crate::LATEST_MAJOR_VERSION)) {
                                        "latest version"
                                    }
                                    "."
                                }
                            }
                        }
                    } else if sidebar_version0.as_deref() != Some(crate::LATEST_MAJOR_VERSION) {
                        template! {
                            div(class="bg-yellow-500 text-white w-full rounded-md mt-4 px-4 py-3") {
                                p { "This is outdated documentation for Sycamore." }
                                p {
                                    "For up-to-date documentation, see the "
                                    a(href=format!("/docs/{}/getting_started/hello_world", crate::LATEST_MAJOR_VERSION)) {
                                        "latest version"
                                    }
                                    "."
                                }
                            }
                        }
                    } else {
                        template! {}
                    })
                    div(ref=docs_container_ref) { "Loading..." }
                }
                div(class="outline flex-none hidden lg:block lg:w-44 fixed right-0 top-0 mt-12") {
                    OutlineView(outline.handle())
                }
            }
        }
    }
}
