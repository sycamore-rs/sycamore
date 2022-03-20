use sycamore::prelude::*;

fn compile_pass<G: GenericNode>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, "Raw text nodes!" };
    
        let _: View<G> = view! { cx,
            p { "First" }
            p { "Second" }
            "Third"
        };
    
        let spliced = 123;
        let _: View<G> = view! { cx, (spliced) };
    });
}

fn main() {}
