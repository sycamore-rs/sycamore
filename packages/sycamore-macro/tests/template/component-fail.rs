use sycamore::prelude::*;

#[component(C<G>)]
fn c() -> Template<G> {
    view! {
        div
    }
}

fn compile_fail<G: Html>() {
    let _: Template<G> = view! { UnknownComponent() };

    let _: Template<G> = view! { C };
    let _: Template<G> = view! { C(1) };
}

fn main() {}
