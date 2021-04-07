

use maple_core::prelude::*;

#[component(Component<G>)]
pub fn component() -> TemplateResult<G> {
    template! {
        div
    }
}

fn compile_pass<G: GenericNode>() {
    let _: TemplateResult<G> = template! { Component() };
}

fn main() {}
