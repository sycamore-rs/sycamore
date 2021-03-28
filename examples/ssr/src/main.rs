#![allow(non_snake_case)]

// use wasm_bindgen::JsCast;
// use web_sys::{Event, HtmlInputElement};
use maple_core::generic_node::vdom;
use maple_core::prelude::*;

fn App<G: GenericNode>() -> TemplateResult<G> {
    let name = Signal::new(String::new());

    let handle_change = cloned!((name) => move |_| unreachable!());

    template! {
        div {
            h1 {
                "Hello "
                ({if !name.get().is_empty() {
                    cloned!((name) => template! {
                        span { (name.get()) }
                    })
                } else {
                    template! { span { "World" } }
                }})
                "!"
            }

            input(placeholder="What is your name?", on:input=handle_change)
        }
    }
}

fn main() {
    let mut app: Option<TemplateResult<vdom::Node>> = None;
    let _owner = create_root(||{
        app = Some(template! { App() });
    });
    println!("{}", app.unwrap().inner_element());
}
