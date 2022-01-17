use serde_lite::Deserialize;
use sycamore::prelude::*;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SidebarItem {
    pub name: String,
    pub href: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SidebarSection {
    pub title: String,
    pub items: Vec<SidebarItem>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SidebarData {
    sections: Vec<SidebarSection>,
}

#[component(Sidebar<G>)]
pub fn sidebar((version, data): (String, SidebarData)) -> View<G> {
    let sections = data
        .sections
        .into_iter()
        .map(|SidebarSection { title, items }| {
            let pages = items
                .into_iter()
                .map(|SidebarItem { name, href }| {
                    view! {
                        li {
                            a(
                                href=format!("../{}", href),
                                class="py-2 sm:py-0 text-sm pl-4 hover:bg-gray-300 dark:hover:bg-gray-700 w-full inline-block rounded transition",
                            ) {
                                (name)
                            }
                        }
                    }
                })
                .collect();

            let pages = View::new_fragment(pages);
            view! {
                li {
                    h1(class="text-lg font-bold py-1 pl-2") {
                        (title)
                    }
                    ul(class="text-gray-700 dark:text-gray-300") {
                        (pages)
                    }
                }
            }
        })
        .collect();

    let sections = View::new_fragment(sections);
    view! {
        ul {
            li {
                a(
                    href="/versions",
                    class="py-2 sm:py-0 text-sm pl-4 font-bold text-gray-700 dark:text-gray-300 \
                    hover:bg-gray-300 dark:hover:bg-gray-700 w-full inline-block rounded transition",
                ) {
                    "Version: " (version)
                }
            }
            (sections)
        }
    }
}
