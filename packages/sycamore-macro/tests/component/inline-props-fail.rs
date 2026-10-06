use sycamore::prelude::*;

#[component(not_inline_props)]
fn NotInlineProps() -> View {
    view! {}
}

#[component]
fn ReceiverProp(self) -> View {
    view! {}
}

struct Foo {
    bar: i32,
}

#[component]
fn PatternWithoutIdent(Foo { bar }: Foo) -> View {
    view! {
        (bar)
    }
}

fn main() {}
