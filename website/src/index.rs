use sycamore::prelude::*;

#[component(Index<G>)]
pub fn index() -> Template<G> {
    template! {
        div(class="flex flex-col items-center w-full mb-10") {
            h1(class="text-5xl font-bold mt-20 mb-5") {
                "Sycamore"
            }

            p(class="mb-10") {
                "Fast isomorphic web apps in Rust + WASM"
            }
            a(
                href="/docs/getting_started/installation",
                class="py-2 px-3 bg-white hover:bg-yellow-500 border-2 border-yellow-500 \
                rounded font-medium transition",
            ) {
                "Read the Book"
            }
        }
        div(class="text-white flex flex-col w-full md:flex-row space-y-4 md:space-y-0 md:space-x-4 mb-10") {
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
                    a(href="/basics/reactivity", class="underline") { "reactive primitives" }
                    " without a cumbersome virtual DOM."
                }
            }
            div(class="bg-yellow-600 md:flex-1 rounded-md p-6") {
                h1(class="text-lg text-center font-semibold mb-3") { "No JavaScript" }
                p(class="mb-2") {
                    "Had enough of JavaScript? So do we."
                }
                p {
                    "Create apps using Sycamore without touching a single line of JS."
                }
            }
        }
    }
}
