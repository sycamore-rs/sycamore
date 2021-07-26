use std::any::TypeId;

use serde_lite::Deserialize;
use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

// Sync definition with docs/build.rs
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MarkdownPage {
    html: String,
    outline: Vec<Outline>,
}

// Sync definition with docs/build.rs
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Outline {
    name: String,
    children: Vec<Outline>,
}

#[wasm_bindgen(inline_js = "export function highlight_element(el) { hljs.highlightElement(el); }")]
extern "C" {
    fn highlight_element(el: &HtmlElement);
}

#[component(OutlineView<G>)]
pub fn outline_view(outline: Vec<Outline>) -> Template<G> {
    template! {
        ul(class="mt-4 text-sm pl-2 border-l border-gray-400") {
            Indexed(IndexedProps {
                iterable: Signal::new(outline).handle(),
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
    pub data: MarkdownPage,
    pub sidebar_version: Option<String>,
}

#[component(Content<G>)]
pub fn content(
    ContentProps {
        pathname,
        data,
        sidebar_version,
    }: ContentProps,
) -> Template<G> {
    let show_sidebar = sidebar_version.is_some();

    let docs_container_ref = NodeRef::new();
    let docs_container = template! {
        div(ref=docs_container_ref)
    };
    if TypeId::of::<G>() == TypeId::of::<DomNode>() {
        let element = docs_container_ref
            .get::<DomNode>()
            .inner_element()
            .unchecked_into::<HtmlElement>();
        element.set_inner_html(&data.html);
        highlight_element(&element);
    }

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
                            div(class="bg-yellow-500 text-white w-full rounded-md mt-4 mb-2 px-4 py-1") {
                                p { "This is unreleased documentation for Sycamore next version." }
                                p {
                                    "For up-to-date documentation, see the "
                                    a(href=format!("/docs/{}/getting_started/installation", crate::LATEST_MAJOR_VERSION)) {
                                        "latest version"
                                    }
                                    " (" (crate::LATEST_MAJOR_VERSION) ")."
                                }
                            }
                        }
                    } else if sidebar_version0.is_some() && sidebar_version0.as_deref() != Some(crate::LATEST_MAJOR_VERSION) {
                        template! {
                            div(class="bg-yellow-500 text-white w-full rounded-md mt-4 mb-2 px-4 py-1") {
                                p { "This is outdated documentation for Sycamore." }
                                p {
                                    "For up-to-date documentation, see the "
                                    a(href=format!("/docs/{}/getting_started/installation", crate::LATEST_MAJOR_VERSION)) {
                                        "latest version"
                                    }
                                    " (" (crate::LATEST_MAJOR_VERSION) ")."

                                }
                            }
                        }
                    } else {
                        template! {}
                    })
                    (docs_container)
                }
                div(class="outline flex-none hidden lg:block lg:w-44 fixed right-0 top-0 mt-12") {
                    OutlineView(data.outline)
                }
            }
        }
    }
}
