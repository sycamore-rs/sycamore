use sycamore::prelude::*;

fn compile_pass<G: GenericNode>() {
    let _: Template<G> = template! { "Raw text nodes!" };

    let _: Template<G> = template! {
        p { "First" }
        p { "Second" }
        "Third"
    };

    let spliced = 123;
    let _: Template<G> = template! { (spliced) };
}

fn main() {}
