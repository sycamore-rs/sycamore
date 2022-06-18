//! Look ma, No `view!`!
//!
//! This example demonstrates the basics of the builder API for constructing views, as an
//! alternative to using the `view!` macro.

use sycamore::builder::prelude::*;
use sycamore::prelude::*;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let name = create_signal(cx, String::new());
    div()
        .c(h1()
            .t("Hello ")
            .dyn_if(
                || !name.get().is_empty(),
                || span().dyn_t(|| name.get().to_string()),
                || span().t("World"),
            )
            .t("!"))
        .c(input().bind_value(name))
        .view(cx)
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|cx| component(|| App(cx)));
}
