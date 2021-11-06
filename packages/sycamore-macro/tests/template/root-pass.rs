use sycamore::prelude::*;

fn compile_pass<G: GenericNode>() {
    let _: Template<G> = view! { "Raw text nodes!" };

    let _: Template<G> = view! {
        p { "First" }
        p { "Second" }
        "Third"
    };

    let spliced = 123;
    let _: Template<G> = view! { (spliced) };
}

fn main() {}
