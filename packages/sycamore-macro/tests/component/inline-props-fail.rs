use sycamore::prelude::*;

#[component(inline_props)]
fn MissingScope<G: Html>() -> View<G> {
    todo!()
}

#[component(inline_props)]
fn ImplTraitInArgs<G: Html>(cx: Scope, foo: impl std::fmt::Display) -> View<G> {
    view! { cx,
        (foo)
    }
}

fn main() {}
