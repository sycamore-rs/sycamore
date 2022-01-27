use serde_lite::Deserialize;
use sycamore::prelude::*;

use crate::sidebar::SidebarData;

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

#[component]
pub fn OutlineView<G: Html>(ctx: ScopeRef, outline: Vec<Outline>) -> View<G> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title("Sycamore"); // TODO: get title from markdown file

    view! { ctx,
        ul(class="mt-4 text-sm pl-2 border-l border-gray-400 dark:border-gray-500 text-gray-600 dark:text-gray-300") {
            Indexed {
                iterable: ctx.create_signal(outline),
                view: |ctx, item| {
                    let Outline { name, children } = item;
                    let nested = children.iter().map(|x| {
                        let name = x.name.clone();
                        let href = format!("#{}", x.name.trim().to_lowercase().replace(" ", "-"));
                        view! { ctx,
                            li {
                                a(
                                    class="hover:text-yellow-400 mb-1 inline-block transition-colors",
                                    href=href,
                                ) {
                                    (name)
                                }
                            }
                        }
                    }).collect();
                    let nested = View::new_fragment(nested);

                    let href = format!("#{}", name.trim().to_lowercase().replace(" ", "-"));

                    view! { ctx,
                        li {
                            a(
                                class="hover:text-yellow-400 mb-1 inline-block transition-colors",
                                href=href,
                            ) {
                                (name)
                            }
                            ul(class="ml-3") {
                                (nested)
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Prop)]
pub struct ContentProps {
    pub data: MarkdownPage,
    #[builder(default, setter(strip_option))]
    pub sidebar: Option<(String, SidebarData)>,
}

#[component]
pub fn Content<G: Html>(
    ctx: ScopeRef,
    ContentProps {
        data: MarkdownPage { html, outline },
        sidebar,
    }: ContentProps,
) -> View<G> {
    let sidebar = ctx.create_ref(sidebar);
    let show_sidebar = sidebar.is_some();

    let sidebar_version = sidebar.as_ref().map(|x| x.0.clone());

    view! { ctx,
        div(class="flex w-full") {
            (if show_sidebar {
                view! { ctx,
                    div(class="flex-none hidden sm:block fixed left-0 top-0 pt-12 max-h-full overflow-y-auto") {
                        div(class="p-3"){
                            crate::sidebar::Sidebar(sidebar.clone().unwrap())
                        }
                    }
                }
            } else {
                view! { ctx, }
            })
            div(class="flex-1 overflow-hidden max-w-screen-xl mx-auto") {
                div(
                    class=format!("content min-w-0 px-4 mb-2 lg:mr-44 {}",
                    if show_sidebar { "sm:ml-44" } else { "container mx-auto lg:ml-auto lg:pr-48" }),
                ) {
                    (if sidebar_version.as_deref() == Some(crate::NEXT_VERSION) {
                        view! { ctx,
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
                    } else if sidebar_version.is_some() && sidebar_version.as_deref() != Some(crate::LATEST_MAJOR_VERSION) {
                        view! { ctx,
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
                        view! { ctx, }
                    })
                    div(dangerously_set_inner_html=&html)
                }
            }
            div(class="outline flex-none hidden lg:block lg:w-44 fixed right-0 top-0 pt-12 max-h-full overflow-y-auto") {
                OutlineView(outline)
            }
        }
    }
}
