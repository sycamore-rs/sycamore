use sycamore::prelude::*;

#[component(Component<G>)]
pub fn component() -> View<G> {
    view! {
        div
    }
}

fn compile_pass<G: Html>() {
    let _: View<G> = view! { Component() };
}

fn main() {}
