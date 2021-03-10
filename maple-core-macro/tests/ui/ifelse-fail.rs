use maple_core::prelude::*;

fn compile_pass() {
    template! {
        div {
            @if unknown_condition {
                p
            }
        }
    };

    template! {
        div {
            @else true {
                p
            }
        }
    };
}

fn main() {}
