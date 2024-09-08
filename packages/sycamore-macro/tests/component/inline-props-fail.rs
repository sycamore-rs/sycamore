use sycamore::prelude::*;

#[component(inline_props)]
fn ImplTraitInArgs(foo: impl std::fmt::Display) -> View {
    view! {
        (foo)
    }
}

#[component(not_inline_props)]
fn NotInlineProps() -> View {
    view! {}
}

fn main() {}
