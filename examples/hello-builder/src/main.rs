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
            .dyn_if(
                || !name.get().is_empty(),
                || h(span).dyn_t(|| name.get().to_string()),
                || h(span).t("World").view(ctx),
            )
            .t("!"))
        .c(h(input).bind_value(name))
        .c(h(button)
            // This svg is from https://fonts.google.com/icons?selected=Material+Icons&icon.query=backspace
            // with the height and width hacked to 12px to make the button look nice.
            // Normally you would do height adjustments like this with CSS though.
            .inner_html(include_str!("./backspace_black_24dp.svg"))
            .t("Clear")
            .on("click", |_| name.set("".to_string())))
        .view(ctx)
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|ctx| component(|| App(ctx, ())));
}
