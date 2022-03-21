//! Look ma, No `view!`!
//!
//! This example demonstrates the basics of the builder API for constructing views, as an
//! alternative to using the `view!` macro.

use sycamore::builder::prelude::*;
use sycamore::prelude::*;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let name = create_signal(cx, String::new());
    h(div)
        .c(h(h1)
            .t("Hello ")
            .dyn_if(
                || !name.get().is_empty(),
                || h(span).dyn_t(|| name.get().to_string()),
                || h(span).t("World"),
            )
            .t("!"))
        .c(h(input).bind_value(name))
        .view(cx)
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|cx| component(|| App(cx, ())));
}
