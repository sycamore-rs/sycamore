use sycamore::prelude::*;

#[component(not_inline_props)]
fn NotInlineProps() -> View {
    view! {}
}

#[component(inline_props)]
fn ReceiverProp(self) -> View {
    view! {}
}

fn main() {}
