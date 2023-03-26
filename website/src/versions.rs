use sycamore::prelude::*;

#[derive(Clone, Copy)]
enum VersionedDocsLink {
    Some(&'static str),
    None,
    Next,
}

const VERSIONS: &[(&str, VersionedDocsLink)] = &[
    ("Next", VersionedDocsLink::Next),
    // v0.9.x
    ("v0.9.0-beta.1", VersionedDocsLink::None),
    // v0.8.x
    ("v0.8.2", VersionedDocsLink::Some("v0.8")),
    ("v0.8.1", VersionedDocsLink::None),
    ("v0.8.0", VersionedDocsLink::None),
    ("v0.8.0-beta.7", VersionedDocsLink::None),
    ("v0.8.0-beta.6", VersionedDocsLink::None),
    ("v0.8.0-beta.5", VersionedDocsLink::None),
    ("v0.8.0-beta.4", VersionedDocsLink::None),
    ("v0.8.0-beta.3", VersionedDocsLink::None),
    ("v0.8.0-beta.2", VersionedDocsLink::None),
    ("v0.8.0-beta.1", VersionedDocsLink::None),
    // v0.7.x
    ("v0.7.1", VersionedDocsLink::Some("v0.7")),
    ("v0.7.0", VersionedDocsLink::None),
    // v0.6.x
    ("v0.6.3", VersionedDocsLink::Some("v0.6")),
    ("v0.6.2", VersionedDocsLink::None),
    ("v0.6.1", VersionedDocsLink::None),
    ("v0.6.0", VersionedDocsLink::None),
    // v0.5.x
    ("v0.5.2", VersionedDocsLink::Some("v0.5")),
    ("v0.5.1", VersionedDocsLink::None),
    ("v0.5.0", VersionedDocsLink::None),
    ("v0.5.0-beta.1", VersionedDocsLink::None),
    ("v0.5.0-beta.0", VersionedDocsLink::None),
];

#[component(inline_props)]
fn VersionedDocsLink<G: Html>(
    cx: Scope,
    name: &'static str,
    versioned_docs_link: VersionedDocsLink,
) -> View<G> {
    match versioned_docs_link {
        VersionedDocsLink::Some(link) => view! { cx,
            a(
                class="hover:text-yellow-500 transition-colors",
                href=format!("/docs/{}/getting_started/installation", link),
            ) { "Book" }
            a(
                class="hover:text-yellow-500 transition-colors",
                href=format!("https://docs.rs/sycamore/{}", &name[1..]),
            ) { "API" }
            a(
                class="hover:text-yellow-500 transition-colors",
                href=format!("https://github.com/sycamore-rs/sycamore/releases/tag/{}", &name[1..]),
            ) { "Release Notes" }
        },
        VersionedDocsLink::None => view! { cx,
            a(
                class="hover:text-yellow-500 transition-colors",
                href=format!("https://docs.rs/sycamore/{}", &name[1..]),
            ) { "API" }
            a(
                class="hover:text-yellow-500 transition-colors",
                href=format!("https://github.com/sycamore-rs/sycamore/releases/tag/{}", &name[1..]),
            ) { "Release Notes" }
        },
        VersionedDocsLink::Next => view! { cx,
            a(
                class="hover:text-yellow-500 transition-colors",
                href="/docs/getting_started/installation",
            ) { "Book" }
            a(
                class="hover:text-yellow-500 transition-colors",
                href="/api/sycamore/index.html",
                rel="external"
            ) { "API" }
        },
    }
}

#[component]
pub fn Versions<G: Html>(cx: Scope) -> View<G> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title("Versions - Sycamore");

    let versions = VERSIONS
        .iter()
        .copied()
        .map(|(name, versioned_docs_link)| {
            view! { cx,
                li {
                    h2(class="text-2xl font-light") { (name) }
                    div(class="flex flex-col divide-y dark:divide-gray-500 text-gray-600 dark:text-gray-300") {
                        VersionedDocsLink(name=name, versioned_docs_link=versioned_docs_link)
                    }
                }
            }
        })
        .collect();
    let versions = View::new_fragment(versions);

    view! { cx,
        div(class="container mx-auto") {
            h1(class="text-4xl font-bold") { "Versions" }
            ul(class="mt-5 ml-2 flex flex-col space-y-4") {
                (versions)
            }
        }
    }
}
