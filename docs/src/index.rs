use maple_core::prelude::*;

#[component(Index<G>)]
pub fn index() -> TemplateResult<G> {
    template! {
        div {
            h1 {
                "Maple"
            }

            a(class="btn btn-primary", href="/getting_started/installation") {
                "Getting started"
            }
        }
    }
}
