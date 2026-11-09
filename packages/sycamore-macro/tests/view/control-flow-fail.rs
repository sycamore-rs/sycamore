use sycamore::prelude::*;

fn compile_fail() {
    let _ = create_root(|| {
        // Wrong type
        let bool_signal = create_signal(true);
        let _: View = view! {
            div {
                if 123 {
                    p {}
                }
            }
            div {
                if bool_signal {
                    p {}
                }
            }
        };
    });
}

fn main() {}
