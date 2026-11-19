use sycamore::prelude::*;

#[component]
pub fn NavLink(href: &'static str, title: &'static str) -> View {
    view! {
        a(class="text-center font-bold text-gray-200 hover:text-gray-100 transition", href=href) {
            (title)
        }
    }
}

#[component]
pub fn Header() -> View {
    view! {
        div(class="bg-green-400 mb-3") {
            nav(class="container flex flex-row space-x-4 mx-auto py-4") {
                NavLink(href="", title="HN")
                NavLink(href="new", title="Latest")
                NavLink(href="best", title="Best")
                NavLink(href="show", title="Show")
                div(class="flex-1")
                a(
                    href="https://sycamore.dev",
                    target="_blank",
                    class="font-light text-white",
                ) { "Built with Sycamore" }
            }
        }
    }
}
