#![allow(non_snake_case)]

mod content;
mod header;
mod index;
mod sidebar;

use maple_core::prelude::*;

fn App<G: GenericNode>() -> TemplateResult<G> {
    let location = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .location()
        .unwrap();
    let pathname = location.pathname().unwrap();

    template! {
        main {
            header::Header()

            (if pathname != "/" {
                template! {
                    content::Content()
                }
            } else {
                template! {
                    div(class="container") {
                        index::Index()
                    }
                }
            })
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
