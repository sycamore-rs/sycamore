use sycamore::prelude::*;

#[component(Component<G>)]
pub fn component() -> Template<G> {
    template! {
        div
    }
}

fn compile_pass<G: Html>() {
    let _: Template<G> = template! { Component() };
}

fn main() {}
