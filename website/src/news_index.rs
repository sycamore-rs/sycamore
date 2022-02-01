use sycamore::prelude::*;

static POSTS: &[(&str, &str, &str)] = &[
    (
        "Announcing Sycamore v0.7.0",
        "Client-side hydration + Builder API",
        "announcing-v0.7.0",
    ),
    (
        "Announcing Sycamore v0.6.0",
        "Faster and faster with plenty of fixes and features...",
        "announcing-v0.6.0",
    ),
    (
        "Announcing Sycamore v0.5.0",
        "SSR + Routing",
        "announcing-v0.5.0",
    ),
];

#[component]
pub fn NewsIndex<G: Html>(ctx: ScopeRef) -> View<G> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title("News - Sycamore");

    let posts = POSTS
        .iter()
        .map(|(title, subtitle, url)| {
            view! { ctx,
                li(class="hover:text-yellow-500 transition-colors") {
                    a(href=format!("/news/{}", url)) {
                        h2(class="text-2xl font-light") { (title) }
                        p(class="text-gray-600 dark:text-gray-400") { (subtitle) }
                    }
                }
            }
        })
        .collect();
    let posts = View::new_fragment(posts);

    view! { ctx,
        div(class="container mx-auto") {
            h1(class="text-4xl font-bold") { "News" }
            ul(class="mt-5 ml-2 space-y-2") {
                (posts)
            }
        }
    }
}
