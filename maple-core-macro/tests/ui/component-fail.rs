#![allow(non_snake_case)]

use maple_core::prelude::*;

fn Component<G: GenericNode>() -> TemplateResult<G> {
    template! {
        div
    }
}

fn compile_fail<G: GenericNode>() {
    let _: TemplateResult<G> = template! { UnknownComponent() };

    let _: TemplateResult<G> = template! { Component };
    let _: TemplateResult<G> = template! { Component(1) };
}

fn main() {}
