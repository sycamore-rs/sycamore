use sycamore::prelude::*;

fn compile_pass<G: GenericNode>() {
    let _: TemplateResult<G> = template! { "Raw text nodes!" };

    let _: TemplateResult<G> = template! {
        p { "First" }
        p { "Second" }
        "Third"
    };

    // let spliced = 123;
    // let _: TemplateResult<G> = template! { (spliced) };
}

fn main() {}
