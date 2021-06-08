use sycamore::prelude::*;

#[component(C<G>)]
fn c() -> Template<G> {
    template! {
        div
    }
}

fn compile_fail<G: GenericNode>() {
    let _: Template<G> = template! { UnknownComponent() };

    let _: Template<G> = template! { C };
    let _: Template<G> = template! { C(1) };
}

fn main() {}
