use sycamore::prelude::*;

#[component(Nav<G>)]
fn nav() -> Template<G> {
    template! {
        nav(class="fixed top-0 z-50 px-8 w-full \
        backdrop-filter backdrop-blur-sm backdrop-saturate-150 bg-opacity-80 \
        bg-gray-100 border-b border-gray-400") {
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
                div(class="flex flex-row ml-2 space-x-4 text-gray-600") {
                    a(class="py-2 px-3 text-sm hover:text-gray-800 hover:underline",
                        href="/docs/getting_started/installation",
                    ) {
                        "Book"
                    }
                    a(class="py-2 px-3 text-sm hover:text-gray-800 hover:underline",
                        href="https://docs.rs/sycamore",
                    ) {
                        "API"
                    }
                    a(class="py-2 px-3 text-sm hover:text-gray-800 hover:underline",
                        href="/news",
                    ) {
                        "News"
                    }
                    a(class="py-2 px-3 text-sm hover:text-gray-800 hover:underline",
                        href="https://github.com/sycamore-rs/sycamore",
                    ) {
                        "GitHub"
                    }
                    a(class="py-2 px-3 text-sm hover:text-gray-800 hover:underline",
                        href="https://discord.gg/vDwFUmm6mU",
                    ) {
                        "Discord"
                    }
                }
            }
        }
    }
}

#[component(Header<G>)]
pub fn header() -> Template<G> {
    template! {
        header {
            Nav()
        }
    }
}
