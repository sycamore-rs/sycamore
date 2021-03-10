#![allow(non_snake_case)]

use maple_core::prelude::*;

pub fn MyComponent(num: StateHandle<i32>) -> TemplateResult {
    template! {
        div(class="my-component") {
            # "My component"
            p {
                # "Value: "
                # num.get()
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let state = Signal::new(1);

    let increment = cloned!((state) => move |_| {
        state.set(*state.get() + 1);
    });

    let root = template! {
        div {
            h1 {
                # "Component demo"
            }

            MyComponent(state.handle())
            MyComponent(state.handle())

            button(on:click=increment) {
                # "Increment"
            }
        }
    };

    render(root);
}
