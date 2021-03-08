#![allow(non_snake_case)]

use maple_core::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let (name, set_name) = create_signal(String::new());

    let displayed_name = create_memo(move || {
        if *name() == "" {
            "World".to_string()
        } else {
            name().as_ref().clone()
        }
    });

    let handle_change = move |event: Event| {
        set_name(
            event
                .target()
                .unwrap()
                .dyn_into::<HtmlInputElement>()
                .unwrap()
                .value(),
        );
    };

    let root = template! {
        div {
            h1 {
                # "Hello "
                # displayed_name()
                # "!"
            }

            input(on:input=handle_change)
        }
    };

    render(root);
}
