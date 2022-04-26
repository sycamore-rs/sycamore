use sycamore::prelude::*;

#[derive(Prop)]
pub struct Prop {
    prop: &'static str,
}

#[component]
pub fn PropComponent<G: Html>(cx: Scope, Prop { prop: _ }: Prop) -> View<G> {
    view! { cx,
        div
    }
}

#[component]
pub fn Component<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div
    }
}

fn compile_pass<G: Html>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, Component() };

        let prop = "prop";
        let _: View<G> = view! { cx, PropComponent { prop: prop } };
        let _: View<G> = view! { cx, PropComponent { prop } };
    });
}

fn main() {}
