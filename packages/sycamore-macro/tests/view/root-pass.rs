use sycamore::prelude::*;

fn compile_pass() {
    let _ = create_root(|| {
        let _: View = view! { "Raw text nodes!" };

        let _: View = view! {
            p { "First" }
            p { "Second" }
            "Third"
        };

        let spliced = 123;
        let _: View = view! { (spliced) };
    });
}

fn main() {}
