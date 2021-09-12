use sycamore::prelude::*;

static POSTS: &[(&str, &str, &str)] = &[
    (
        "Announcing Sycamore v0.5.0",
        "SSR + Routing",
        "announcing-v0.5.0",
    ),
    (
        "Announcing Sycamore v0.6.0",
        "Faster and faster with plenty of fixes and features...",
        "announcing-v0.6.0",
    ),
];

#[component(NewsIndex<G>)]
pub fn news_index() -> Template<G> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title("News - Sycamore");

    let posts = POSTS
        .iter()
        .map(|(title, subtitle, url)| {
            template! {
                li(class="hover:text-yellow-500 transition-colors") {
                    a(href=format!("/news/{}", url)) {
                        h2(class="text-2xl font-light") { (title) }
                        p(class="text-gray-600 dark:text-gray-400") { (subtitle) }
                    }
                }
            }
        })
        .collect();
    let posts = Template::new_fragment(posts);

    template! {
        div(class="container mx-auto") {
            h1(class="text-4xl font-bold") { "News" }
            ul(class="mt-5 ml-2 space-y-2") {
                (posts)
            }
        }
    }
}
