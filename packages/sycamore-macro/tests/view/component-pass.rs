use sycamore::prelude::*;

#[component]
pub fn Component<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div {}
    }
}

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

#[derive(Props)]
pub struct AllDefaultProps {
    #[prop(default)]
    prop: u32,
}

#[component]
pub fn AllDefaultPropsComponent<G: Html>(cx: Scope, _props: AllDefaultProps) -> View<G> {
    view! { cx,
        div {}
    }
}

#[derive(Props)]
pub struct PropsWithChildren<'a, G: GenericNode> {
    children: Children<'a, G>,
}

#[component]
pub fn ComponentWithChildren<'a, G: Html>(cx: Scope<'a>, props: PropsWithChildren<'a, G>) -> View<G> {
    let children = props.children.call(cx);

    view! { cx,
        div {
            (children)
        }
    }
}

#[component]
pub fn NestedComponentWithChildren<'a, G: Html>(cx: Scope<'a>, props: PropsWithChildren<'a, G>) -> View<G> {
    let children = props.children.call(cx);

    view! { cx,
        ComponentWithChildren {
            (children)
            Component {}
        }
    }
}

#[component]
pub async fn AsyncComponentWithPropsDestructuring<'a, G: Html>(
    cx: Scope<'a>,
    PropsWithChildren { children }: PropsWithChildren<'a, G>,
) -> View<G> {
    children.call(cx)
}

fn compile_pass<G: Html>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, Component() };
        let _: View<G> = view! { cx, Component {} };

        let prop = "prop";
        let _: View<G> = view! { cx, PropsComponent(prop=prop) };

        let _: View<G> = view! { cx, AllDefaultPropsComponent(prop=123) };
        let _: View<G> = view! { cx, AllDefaultPropsComponent() };
        let _: View<G> = view! { cx, AllDefaultPropsComponent {} };

        let _: View<G> = view! { cx, ComponentWithChildren { Component() } };
        let _: View<G> = view! { cx, ComponentWithChildren { div {} } };
        let _: View<G> = view! { cx, ComponentWithChildren { div {} div {} } };
        let _: View<G> = view! { cx, ComponentWithChildren { Component {} } };
        let _: View<G> = view! { cx, ComponentWithChildren() { Component {} } };
        let _: View<G> = view! { cx, ComponentWithChildren {} };
        let _: View<G> = view! { cx, ComponentWithChildren() };
        let _: View<G> = view! { cx, ComponentWithChildren() {} };

        let _: View<G> = view! { cx,
            AsyncComponentWithPropsDestructuring {
                Component {}
            }
        };

        let _: View<G> = view! { cx,
            NestedComponentWithChildren {
                Component {}
            }
        };
    });
}

fn main() {}
