mod content;
mod header;
mod index;
mod sidebar;

use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> Template<G> {
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

            div(class="mt-12") {
                (if pathname != "/" {
                    template! {
                        content::Content()
                    }
                } else {
                    template! {
                        div(class="container mx-auto") {
                            index::Index()
                        }
                    }
                })
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
