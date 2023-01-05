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
pub struct OptionalProps {
    #[prop(default, setter(strip_option))]
    optional: Option<u32>,
    implicit: Option<u32>,
}

#[component]
pub fn OptionalPropsComponent<G: Html>(cx: Scope, _props: OptionalProps) -> View<G> {
    view! { cx,
        div {}
    }
}

#[derive(Props)]
pub struct PropsWithChildren<'a, G: GenericNode> {
    children: Children<'a, G>,
}

#[component]
pub fn ComponentWithChildren<'a, G: Html>(
    cx: Scope<'a>,
    props: PropsWithChildren<'a, G>,
) -> View<G> {
    let children = props.children.call(cx);

    view! { cx,
        div {
            (children)
        }
    }
}

#[component]
pub fn NestedComponentWithChildren<'a, G: Html>(
    cx: Scope<'a>,
    props: PropsWithChildren<'a, G>,
) -> View<G> {
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

#[derive(Props)]
pub struct AttributesProps<'cx, G: Html> {
    attributes: Attributes<'cx, G>,
}

#[component]
pub fn AttributesComponent<'cx, G: Html>(
    cx: Scope<'cx>,
    AttributesProps { mut attributes }: AttributesProps<'cx, G>,
) -> View<G> {
    view! { cx,
        input(..attributes) {}
    }
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

        let _: View<G> = view! { cx, OptionalPropsComponent(optional=123) };
        let _: View<G> = view! { cx, OptionalPropsComponent(implicit=123) };
        let _: View<G> = view! { cx, OptionalPropsComponent(optional=123, implicit=123) };
        let _: View<G> = view! { cx, OptionalPropsComponent() };
        let _: View<G> = view! { cx, OptionalPropsComponent {} };

        let _: View<G> = view! { cx, ComponentWithChildren { Component() } };
        let _: View<G> = view! { cx, ComponentWithChildren { div {} } };
        let _: View<G> = view! { cx, ComponentWithChildren { div {} div {} } };
        let _: View<G> = view! { cx, ComponentWithChildren { Component {} } };
        let _: View<G> = view! { cx, ComponentWithChildren() { Component {} } };
        let _: View<G> = view! { cx, ComponentWithChildren {} };
        let _: View<G> = view! { cx, ComponentWithChildren() };
        let _: View<G> = view! { cx, ComponentWithChildren() {} };
        let _: View<G> = view! { cx, AttributesComponent(attr:class = "test") {} };
        let str_signal = create_signal(cx, String::new());
        let _: View<G> = view! { cx, AttributesComponent(bind:value = str_signal) {} };
        let on_click = |_| {};
        let _: View<G> = view! { cx, AttributesComponent(on:click = on_click) {} };

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
