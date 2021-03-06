use maple_core::prelude::*;

fn main() {
    console_error_panic_hook::set_once();

    let root = template! {
        div {
            # "Counter demo"
            p(class="value") {
                # "Value: "
                # 0
            }
            button(class="increment") {
                # "Increment"
            }
            button(class="reset") {
                # "Reset"
            }
        }
    };

    render(&root);
}
