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
fn Component<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div
    }
}

fn compile_fail<G: Html>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, UnknownComponent() };
        let _: View<G> = view! { cx, UnknownComponent {} };

        let _: View<G> = view! { cx, Component };
        let _: View<G> = view! { cx, Component(1) };

        let _: View<G> = view! { cx, PropComponent() };
        let _: View<G> = view! { cx, PropComponent {} };
        let _: View<G> = view! { cx, PropComponent { prop: 123 } };
    });
}

fn main() {}
