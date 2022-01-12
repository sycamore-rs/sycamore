use sycamore::prelude::*;

#[component(Index<G>)]
pub fn index() -> View<G> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title("Sycamore");

    view! {
        div(class="pb-10 mx-4") {
            div(class="flex flex-col items-center w-full mb-10") {
                h1(class="text-5xl font-bold mt-20 mb-5") {
                    "Sycamore"
                }

                p(class="mb-5 text-center") {
                    "A reactive library for creating web apps in Rust and WebAssembly"
                }

                // region: badges
                div(class="mb-7 flex flex-row flex-wrap justify-center gap-1") {
                    a(
                        href="https://github.com/sycamore-rs/sycamore",
                        target="_blank"
                    ) { img(src="https://img.shields.io/github/stars/sycamore-rs/sycamore?style=social", alt="GitHub Stars") }
                    a(
                        href="https://crates.io/crates/sycamore",
                        target="_blank"
                    ) { img(src="https://img.shields.io/crates/v/sycamore", alt="Crates.io") }
                    a(
                        href="https://docs.rs/sycamore",
                        target="_blank"
                    ) { img(src="https://img.shields.io/docsrs/sycamore?color=blue&label=docs.rs", alt="docs.rs") }
                    a(
                        href="https://github.com/sycamore-rs/sycamore/graphs/contributors",
                        target="_blank"
                    ) { img(src="https://img.shields.io/github/contributors/sycamore-rs/sycamore", alt="Github Contributors") }
                    a(
                        href="https://discord.gg/vDwFUmm6mU",
                        target="_blank"
                    ) { img(src="https://img.shields.io/discord/820400041332179004?label=discord", alt="Discord") }
                }
                // endregion

                a(
                    href="/docs/getting_started/installation",
                    class="py-2 px-3 text-white bg-yellow-600 rounded font-medium transition whitespace-nowrap",
                ) {
                    "Read the Book"
                }
            }
            div(class="text-white flex flex-col w-full md:flex-row space-y-4 md:space-y-0 md:space-x-4") {
                div(class="bg-red-500 md:flex-1 rounded-md p-6") {
                    h1(class="text-lg text-center font-semibold mb-3") { "Lightning speed" }
                    p {
                        "Sycamore harnesses the full power of "
                        a(href="https://www.rust-lang.org/", class="underline", target="_blank") { "Rust" }
                        " via "
                        a(href="https://webassembly.org/", class="underline", target="_blank") { "WebAssembly" }
                        ", giving you full \
                        control over performance."
                    }
                }
                div(class="bg-amber-600 md:flex-1 rounded-md p-6") {
                    h1(class="text-lg text-center font-semibold mb-3") { "Ergonomic and intuitive" }
                    p {
                        "Write code that feels natural. Everything is built on "
                        a(href="/docs/basics/reactivity", class="underline") { "reactive primitives" }
                        " without a cumbersome virtual DOM."
                    }
                }
                div(class="bg-yellow-600 md:flex-1 rounded-md p-6") {
                    h1(class="text-lg text-center font-semibold mb-3") { "No JavaScript" }
                    p(class="mb-2") {
                        "Had enough of JavaScript? So have we."
                    }
                    p {
                        "Create apps using Sycamore without touching a single line of JS."
                    }
                }
            }
        }
    }
}
