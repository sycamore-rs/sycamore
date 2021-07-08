use sycamore::prelude::*;

#[component(Versions<G>)]
pub fn versions() -> Template<G> {
    template! {
        div(class="container mx-auto") {
            h1(class="text-4xl font-bold") { "Versions" }
        }
    }
}
