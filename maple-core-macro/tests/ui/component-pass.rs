#![allow(non_snake_case)]

use maple_core::prelude::*;

pub fn Component<G: GenericNode>() -> TemplateResult<G> {
    template! {
        div
    }
}

fn compile_pass<G: GenericNode>() {
    let _: TemplateResult<G> = template! { Component() };
}

fn main() {}
