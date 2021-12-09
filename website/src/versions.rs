use sycamore::prelude::*;

enum VersionedDocsLink {
    Some(&'static str),
    None,
    Next,
}

const VERSIONS: &[(&str, VersionedDocsLink)] = &[
    ("Next", VersionedDocsLink::Next),
    // v0.7.x
    ("v0.7.0", VersionedDocsLink::Some("v0.7")),
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

#[component(VersionedDocsLinkView<G>)]
fn versioned_docs_link_view(
    (name, versioned_docs_link): (&'static str, &'static VersionedDocsLink),
) -> View<G> {
    match versioned_docs_link {
        VersionedDocsLink::Some(link) => view! {
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
        VersionedDocsLink::None => view! {
            a(
                class="hover:text-yellow-500 transition-colors",
                href=format!("https://docs.rs/sycamore/{}", &name[1..]),
            ) { "API" }
            a(
                class="hover:text-yellow-500 transition-colors",
                href=format!("https://github.com/sycamore-rs/sycamore/releases/tag/{}", &name[1..]),
            ) { "Release Notes" }
        },
        VersionedDocsLink::Next => view! {
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

#[component(Versions<G>)]
pub fn versions() -> View<G> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title("Versions - Sycamore");

    let versions = VERSIONS
        .iter()
        .map(|(name, versioned_docs_link)| {
            view! {
                li {
                    h2(class="text-2xl font-light") { (name) }
                    div(class="flex flex-col divide-y dark:divide-gray-500 text-gray-600 dark:text-gray-300") {
                        VersionedDocsLinkView((name, versioned_docs_link))
                    }
                }
            }
        })
        .collect();
    let versions = View::new_fragment(versions);

    view! {
        div(class="container mx-auto") {
            h1(class="text-4xl font-bold") { "Versions" }
            ul(class="mt-5 ml-2 flex flex-col space-y-4") {
                (versions)
            }
        }
    }
}
