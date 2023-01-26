//! Look ma, No `view!`!
//!
//! This example demonstrates the basics of the builder API for constructing views, as an
//! alternative to using the `view!` macro.

use sycamore::builder::*;
use sycamore::prelude::*;

#[component]
fn App(cx: Scope) -> View {
    let name = create_signal(cx, String::new());
    div(cx)
        .child(
            p(cx)
                .child("Hello ")
                .dyn_child(|cx| {
                    if name.get().is_empty() {
                        span(cx).child("World").view()
                    } else {
                        span(cx).dyn_child(|_| name.get().to_string()).view()
                    }
                })
                .child("!"),
        )
        .child(input(cx).bind(bind::value, name))
        .view()
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(App);
}
