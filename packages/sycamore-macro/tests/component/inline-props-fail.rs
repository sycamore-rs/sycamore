use sycamore::prelude::*;

#[component(not_inline_props)]
fn NotInlineProps() -> View {
    view! {}
}

#[component(inline_props)]
fn ReceiverProp(self) -> View {
    view! {}
}

struct Foo {
    bar: i32,
}

#[component(inline_props)]
fn PatternWithoutIdent(Foo { bar }: Foo) -> View {
    view! {
        (bar)
    }
}

fn main() {}
