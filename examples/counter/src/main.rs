use maple_core::prelude::*;

fn main() {
    console_error_panic_hook::set_once();

    let root = template! {
        div {
            p(class="value")
            button(class="increment")
            button(class="reset")
        }
    };

    render(root);
}
