//! Look ma, No `view!`!
//!
//! This example demonstrates the basics of the builder API for constructing views, as an
//! alternative to using the `view!` macro.

use sycamore::builder::prelude::*;
use sycamore::prelude::*;

#[component]
fn App<G: Html>(ctx: ScopeRef) -> View<G> {
    let name = ctx.create_signal(String::new());
    h(div)
        .c(h(h1)
            .t("Hello ")
            .dyn_c(|| {
                if *ctx.create_selector(|| !name.get().is_empty()).get() {
                    h(span).dyn_t(|| name.get().to_string()).view(ctx)
                } else {
                    h(span).t("World").view(ctx)
                }
            })
            .t("!"))
        .c(h(input).bind_value(name))
        .view(ctx)
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|ctx| component(|| App(ctx, ())));
}
