use sycamore::prelude::*;

#[component(Component<G>)]
pub fn component() -> Template<G> {
    view! {
        div
    }
}

fn compile_pass<G: Html>() {
    let _: Template<G> = view! { Component() };
}

fn main() {}
