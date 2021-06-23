use sycamore::prelude::*;
use sycamore_router::Link;

#[component(Nav<G>)]
fn nav() -> Template<G> {
    template! {
        nav(class="fixed top-0 z-50 px-8 w-full \
        backdrop-filter backdrop-blur-sm backdrop-saturate-150 bg-opacity-80 \
        bg-gray-100 border-b border-gray-400") {
            div(class="flex flex-row justify-between items-center h-12") {
                // Brand section
                div(class="flex-initial") {
                    div(class="flex space-x-4 text-white") {
                        Link(("/", template! {
                            span(class="py-2 px-3 text-sm font-medium \
                            bg-gray-500 hover:bg-gray-600 transition-colors rounded", href="/#") {
                                "Sycamore"
                            }
                        }))
                    }
                }
                // Links section
                div(class="flex flex-row ml-2 space-x-4 text-white") {
                    a(class="py-2 px-3 text-sm text-gray-600 hover:text-gray-800 hover:underline transition",
                        href="/getting_started/installation",
                    ) {
                        "Book"
                    }
                    a(class="py-2 px-3 text-sm text-gray-600 hover:text-gray-800 hover:underline transition",
                        href="https://docs.rs/sycamore",
                    ) {
                        "API"
                    }
                    a(class="py-2 px-3 text-sm text-gray-600 hover:text-gray-800 hover:underline transition",
                        href="https://github.com/sycamore-rs/sycamore",
                    ) {
                        "Repository"
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
