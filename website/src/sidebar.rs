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

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Sidebar {
    pub version: String,
    pub path: String,
    pub data: SidebarData,
}

#[component(inline_props)]
pub fn Sidebar<G: Html>(sidebar: Sidebar) -> View<G> {
    let sections = sidebar.data
        .sections
        .into_iter()
        .map(|SidebarSection { title, items }| {
            let pages = items
                .into_iter()
                .map(|SidebarItem { name, href }| {
                    let selected = if href == sidebar.path{"font-bold underline"}else{""};
                    let class = format!("py-2 sm:py-0 text-sm pl-4 hover:bg-gray-300 dark:hover:bg-gray-700 w-full inline-block rounded transition {}", selected);
                    view! {
                        li {
                            a(
                                href=format!("../{}", href),
                                class=class,
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
                    "Version: " (sidebar.version)
                }
            }
            (sections)
        }
    }
}
