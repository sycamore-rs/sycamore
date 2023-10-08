//! Look ma, No `view!`!
//!
//! This example demonstrates the basics of the builder API for constructing views, as an
//! alternative to using the `view!` macro.

use sycamore::builder::prelude::*;
use sycamore::prelude::*;

#[component]
fn App<G: Html>() -> View<G> {
    let name = create_signal(String::new());
    div()
        .c(h1()
            .t("Hello ")
            .dyn_if(
                move || !name.with(String::is_empty),
                move || span().dyn_t(move || name.get_clone()),
                move || span().t("World"),
            )
            .t("!"))
        .c(input().bind_value(name))
        .view()
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(App);
}
