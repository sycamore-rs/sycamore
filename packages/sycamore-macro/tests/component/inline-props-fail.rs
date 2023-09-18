use sycamore::prelude::*;

#[component(inline_props)]
fn ImplTraitInArgs<G: Html>(foo: impl std::fmt::Display) -> View<G> {
    view! {
        (foo)
    }
}

#[component(not_inline_props)]
fn NotInlineProps<G: Html>() -> View<G> {
    view! {}
}

fn main() {}
