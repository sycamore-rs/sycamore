use sycamore::prelude::*;

enum VersionedDocsLink {
    Some(&'static str),
    None,
    Next,
}

const VERSIONS: &[(&str, VersionedDocsLink)] = &[
    ("Next", VersionedDocsLink::Next),
    ("v0.5.2", VersionedDocsLink::Some("v0.5")),
    ("v0.5.1", VersionedDocsLink::None),
    ("v0.5.0", VersionedDocsLink::None),
    ("v0.5.0-beta.1", VersionedDocsLink::None),
    ("v0.5.0-beta.0", VersionedDocsLink::None),
];

#[component(VersionedDocsLinkView<G>)]
fn versioned_docs_link_view(
    (name, versioned_docs_link): (&'static str, &'static VersionedDocsLink),
) -> Template<G> {
    match versioned_docs_link {
        VersionedDocsLink::Some(link) => template! {
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
        VersionedDocsLink::None => template! {
            a(
                class="hover:text-yellow-500 transition-colors",
                href=format!("https://docs.rs/sycamore/{}", &name[1..]),
            ) { "API" }
            a(
                class="hover:text-yellow-500 transition-colors",
                href=format!("https://github.com/sycamore-rs/sycamore/releases/tag/{}", &name[1..]),
            ) { "Release Notes" }
        },
        VersionedDocsLink::Next => template! {
            a(
                class="hover:text-yellow-500 transition-colors",
                href="/docs/getting_started/installation",
            ) { "Book" }
        },
    }
}

#[component(Versions<G>)]
pub fn versions() -> Template<G> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title("Versions - Sycamore");

    let versions = VERSIONS
        .iter()
        .map(|(name, versioned_docs_link)| {
            template! {
                li {
                    h2(class="text-2xl font-light") { (name) }
                    div(class="flex flex-col divide-y dark:divide-gray-500 text-gray-600 dark:text-gray-300") {
                        VersionedDocsLinkView((name, versioned_docs_link))
                    }
                }
            }
        })
        .collect();
    let versions = Template::new_fragment(versions);

    template! {
        div(class="container mx-auto") {
            h1(class="text-4xl font-bold") { "Versions" }
            ul(class="mt-5 ml-2 flex flex-col space-y-4") {
                (versions)
            }
        }
    }
}
