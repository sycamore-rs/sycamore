use sycamore::prelude::*;

#[component]
pub fn Index<G: Html>(cx: Scope) -> View<G> {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .set_title("Sycamore");

    view! { cx,
        div(class="pb-10 mx-4") {
            div(class="flex flex-col items-center w-full mb-10") {
                img(src="/logo.svg", alt="logo", class="w-32 h-32 mt-10")

                h1(class="text-5xl font-extrabold pb-5 \
                    bg-gradient-to-r from-orange-300 to-red-400 text-transparent bg-clip-text") {
                    "Sycamore"
                }

                p(class="font-mono mb-5 text-center") {
                    "A " span(class="underline") { "reactive" }
                    " library for creating web apps in " span(class="underline") { "Rust" }
                    " and " span(class="underline") { "WebAssembly" }
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

                div {
                    a(
                        href="/docs/getting_started/installation",
                        class="inline-block py-2.5 px-5 text-lg font-medium text-white bg-orange-500 rounded-lg \
                            transition hover:bg-red-400 hover:-translate-y-0.5"
                    ) {
                        "Get Started"
                    }
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
