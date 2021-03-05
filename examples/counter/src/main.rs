use maple_core::prelude::*;

fn main() {
    let root = template! {
        p(class="test", id="my-paragraph")
    };

    render(root);
}
