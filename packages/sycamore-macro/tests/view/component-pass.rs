use sycamore::prelude::*;

#[component]
pub fn Component<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div
    }
}

fn compile_pass<G: Html>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, Component() };
    });
}

fn main() {}
