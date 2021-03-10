use maple_core::prelude::*;

fn compile_pass() {
    let condition = true;

    template! {
        @if condition {
            p
        }
    };

    template! {
        div {
            @if condition {
                p
            }
        }
    };
}

fn main() {}
