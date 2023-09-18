use sycamore::prelude::*;

fn compile_pass<G: Html>() {
    let _ = create_root(|| {
        let _: View<G> = view! { "Raw text nodes!" };

        let _: View<G> = view! {
            p { "First" }
            p { "Second" }
            "Third"
        };

        let spliced = 123;
        let _: View<G> = view! { (spliced) };
    });
}

fn main() {}
