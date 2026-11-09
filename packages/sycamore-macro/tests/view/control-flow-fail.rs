use sycamore::prelude::*;

fn compile_fail() {
    let _ = create_root(|| {
        // Wrong type
        let _: View = view! {
            div {
                if 123 {
                    p {}
                }
            }
        };
    });
}

fn main() {}
