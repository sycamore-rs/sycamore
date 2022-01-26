use sycamore::prelude::*;

#[component]
fn c(ctx: ScopeRef) -> View<G> {
    view! {
        div
    }
}

fn compile_fail<G: Html>() {
    create_scope_immediate(|ctx| {
        let _: View<G> = view! { ctx, UnknownComponent() };

        let _: View<G> = view! { ctx, C };
        let _: View<G> = view! { ctx, C(1) };
    });
}

fn main() {}
