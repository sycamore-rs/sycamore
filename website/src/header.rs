use sycamore::context::use_context;
use sycamore::prelude::*;

use crate::DarkMode;

#[component(Nav<G>)]
fn nav() -> Template<G> {
    static LINK_CLASS: &str =
        "py-2 px-3 text-sm hover:text-gray-800 dark:hover:text-gray-100 hover:underline";

    let dark_mode = use_context::<DarkMode>();

    template! {
        nav(class="px-8 backdrop-filter backdrop-blur-sm backdrop-saturate-150 bg-opacity-80 \
        bg-gray-100 dark:bg-gray-800 border-b border-gray-400 dark:border-gray-600") {
            div(class="flex flex-row justify-between items-center h-12") {
                // Brand section
                div(class="flex-initial") {
                    div(class="flex space-x-4") {
                        a(href="/#", class="py-2 px-3 text-sm text-white font-medium \
                        bg-gray-500 hover:bg-gray-600 transition-colors rounded") {
                            "Sycamore"
                        }
                    }
                }
                // Links section
                div(class="flex flex-row ml-2 space-x-4 text-gray-600 dark:text-gray-300") {
                    a(class=LINK_CLASS, href="/docs/getting_started/installation",
                    ) {
                        "Book"
                    }
                    a(class=LINK_CLASS, href="https://docs.rs/sycamore",
                    ) {
                        "API"
                    }
                    a(class=LINK_CLASS, href="/news",
                    ) {
                        "News"
                    }
                    a(class=LINK_CLASS, href="https://github.com/sycamore-rs/sycamore",
                    ) {
                        "GitHub"
                    }
                    a(class=LINK_CLASS, href="https://discord.gg/vDwFUmm6mU",
                    ) {
                        "Discord"
                    }
                    button(on:click=move |_| dark_mode.0.set(!*dark_mode.0.get())) {
                        "Toggle dark mode"
                    }
                }
            }
        }
    }
}

#[component(Header<G>)]
pub fn header() -> Template<G> {
    template! {
        header(class="fixed top-0 z-50 w-full") {
            Nav()
        }
    }
}
