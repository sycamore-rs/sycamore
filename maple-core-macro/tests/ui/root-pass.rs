use maple_core::prelude::*;

fn compile_pass<G: GenericNode>() {
    let _: TemplateResult<G> = template! { "Raw text nodes!" };

    // let spliced = 123;
    // let _: TemplateResult<G> = template! { (spliced) };
}

fn main() {}
