use sycamore::prelude::*;

#[component]
fn c(cx: Scope) -> View<G> {
    view! {
        div
    }
}

fn compile_fail<G: Html>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, UnknownComponent() };

        let _: View<G> = view! { cx, C };
        let _: View<G> = view! { cx, C(1) };
    });
}

fn main() {}
