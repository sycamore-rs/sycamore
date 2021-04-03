#![allow(non_snake_case)]

use maple_core::prelude::*;

fn ListItem<G: GenericNode>(value: i32) -> TemplateResult<G> {
    template! {
        p {
            span(class="placeholder")
            i { (value) }
            button(class="delete") {
                i(class="delete-icon")
            }
        }
    }
}

fn App<G: GenericNode>() -> TemplateResult<G> {
    let values = Signal::new((0i32..=10).collect::<Vec<_>>());

    template! {
        div(class="my-container") {
            Indexed(IndexedProps {
                iterable: values.handle(),
                template: |x| template! {
                    // ListItem(x)
                    p { (x) }
                }
            })
        }
    }
}

fn main() {
    let s = render_to_string(|| template! { App() });
    println!("{}", s);
}
