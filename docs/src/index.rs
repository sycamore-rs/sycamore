use sycamore::prelude::*;

#[component(Index<G>)]
pub fn index() -> Template<G> {
    template! {
        div {
            h1 {
                "Sycamore"
            }

            a(class="btn btn-primary", href="/getting_started/installation") {
                "Read the Book!"
            }
        }
    }
}
