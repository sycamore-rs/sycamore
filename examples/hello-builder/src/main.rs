use sycamore::builder::html::*;
use sycamore::prelude::*;

#[component]
fn App<G: Html>(ctx: ScopeRef) -> View<G> {
    let name = ctx.create_signal(String::new());

    div(ctx)
        .child(
            h1(ctx)
                .text("Hello ")
                .dyn_child(move || {
                    if *ctx.create_selector(move || !name.get().is_empty()).get() {
                        span(ctx).dyn_text(move || name.get().to_string()).build()
                    } else {
                        span(ctx).text("World").build()
                    }
                })
                .text("!")
                .build(),
        )
        .child(input(ctx).bind_value(name).build())
        .build()
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|ctx| view! {ctx, App() });
}
