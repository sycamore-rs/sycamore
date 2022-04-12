use sycamore::prelude::*;

#[derive(Prop)]
pub struct Prop {
    value: &'static str,
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
        let _: View<G> = view! { cx, Component {} };

        let value = "prop";
        let _: View<G> = view! { cx, PropComponent { value: value } };
        let _: View<G> = view! { cx, PropComponent { value } };
    });
}

fn main() {}
