use maple_core::prelude::*;

fn main() {
    console_error_panic_hook::set_once();

    let id = "my-paragraph";

    let root = template! {
        p(class="test", id=id) {
            p(class="test")
        }
    };

    render(root);
}
