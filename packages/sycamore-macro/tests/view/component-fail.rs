use sycamore::prelude::*;

#[derive(Props)]
pub struct Props {
    prop: &'static str,
}

#[component]
pub fn PropsComponent<G: Html>(cx: Scope, Props { prop: _ }: Props) -> View<G> {
    view! { cx,
        div {}
    }
}


#[component]
fn Component<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div {}
    }
}

fn compile_fail<G: Html>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, UnknownComponent() };
        let _: View<G> = view! { cx, UnknownComponent {} };

        let _: View<G> = view! { cx, Component };
        let _: View<G> = view! { cx, Component(1) };

        let _: View<G> = view! { cx, PropsComponent() };
        let _: View<G> = view! { cx, PropsComponent {} };
        let _: View<G> = view! { cx, PropsComponent(prop=123) };
        let _: View<G> = view! { cx, PropsComponent { prop: "123" } }; // Legacy syntax.
    });
}

fn main() {}
