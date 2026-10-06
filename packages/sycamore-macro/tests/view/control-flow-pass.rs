use sycamore::prelude::*;

fn compile_pass() {
    let _ = create_root(|| {
        // Static with if/else
        let _: View = view! {
            div {
                if true {
                    p {}
                } else {
                    p {}
                }
            }
        };
        // Static with if only
        let _: View = view! {
            div {
                if true {
                    p {}
                }
            }
        };
        // Static with if/else/else if
        let _: View = view! {
            div {
                if true {
                    p {}
                } else if false {
                    p {}
                } else {
                    p {}
                }
            }
        };
    });
}

fn main() {}
