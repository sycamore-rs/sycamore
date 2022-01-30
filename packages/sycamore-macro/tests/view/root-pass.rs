use sycamore::prelude::*;

fn compile_pass<G: GenericNode>() {
    create_scope_immediate(|ctx| {
        let _: View<G> = view! { ctx, "Raw text nodes!" };
    
        let _: View<G> = view! { ctx,
            p { "First" }
            p { "Second" }
            "Third"
        };
    
        let spliced = 123;
        let _: View<G> = view! { ctx, (spliced) };
    });
}

fn main() {}
