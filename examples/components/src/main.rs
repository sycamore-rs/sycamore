#![allow(non_snake_case)]

use std::rc::Rc;

use maple_core::prelude::*;

pub fn MyComponent(num: Rc<impl Fn() -> Rc<i32> + 'static>) -> HtmlElement {
    template! {
        div(class="my-component") {
            # "My component"
            p {
                # "Value: "
                # num()
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let (state, set_state) = create_signal(1);

    let increment = {
        let state = state.clone();
        let set_state = set_state.clone();
        move || {
            set_state(*state() + 1);
        }
    };

    let root = template! {
        div {
            h1 {
                # "Component demo"
            }

            MyComponent(state.clone())
            MyComponent(state.clone())

            button(on:click=increment) {
                # "Increment"
            }
        }
    };

    render(root);
}
