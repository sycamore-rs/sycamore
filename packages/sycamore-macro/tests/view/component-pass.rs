use sycamore::prelude::*;

#[component]
pub fn Component<G: Html>() -> View<G> {
    view! {
        div {}
    }
}

#[derive(Props)]
pub struct Props {
    prop: &'static str,
}

#[component]
pub fn PropsComponent<G: Html>(Props { prop: _ }: Props) -> View<G> {
    view! {
        div {}
    }
}

#[derive(Props)]
pub struct AllDefaultProps {
    #[prop(default)]
    prop: u32,
}

#[component]
pub fn AllDefaultPropsComponent<G: Html>(_props: AllDefaultProps) -> View<G> {
    view! {
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
pub fn OptionalPropsComponent<G: Html>(_props: OptionalProps) -> View<G> {
    view! {
        div {}
    }
}

#[derive(Props)]
pub struct PropsWithChildren<G: GenericNode> {
    children: Children<G>,
}

#[component]
pub fn ComponentWithChildren<G: Html>(props: PropsWithChildren<G>) -> View<G> {
    let children = props.children.call();

    view! {
        div {
            (children)
        }
    }
}

#[component]
pub fn NestedComponentWithChildren<G: Html>(props: PropsWithChildren<G>) -> View<G> {
    let children = props.children.call();

    view! {
        ComponentWithChildren {
            (children)
            Component {}
        }
    }
}

#[component]
pub async fn AsyncComponentWithPropsDestructuring<G: Html>(
    PropsWithChildren { children }: PropsWithChildren<G>,
) -> View<G> {
    children.call()
}

#[derive(Props)]
pub struct AttributesProps<G: Html> {
    attributes: Attributes<G>,
}

#[component]
pub fn AttributesComponent<G: Html>(AttributesProps { attributes }: AttributesProps<G>) -> View<G> {
    view! {
        input(..attributes) {}
    }
}

fn compile_pass<G: Html>() {
    let _ = create_root(|| {
        let _: View<G> = view! { Component() };
        let _: View<G> = view! { Component {} };

        let prop = "prop";
        let _: View<G> = view! { PropsComponent(prop=prop) };

        let _: View<G> = view! { AllDefaultPropsComponent(prop=123) };
        let _: View<G> = view! { AllDefaultPropsComponent() };
        let _: View<G> = view! { AllDefaultPropsComponent {} };

        let _: View<G> = view! { OptionalPropsComponent(optional=123) };
        let _: View<G> = view! { OptionalPropsComponent(implicit=123) };
        let _: View<G> = view! { OptionalPropsComponent(optional=123, implicit=123) };
        let _: View<G> = view! { OptionalPropsComponent() };
        let _: View<G> = view! { OptionalPropsComponent {} };

        let _: View<G> = view! { ComponentWithChildren { Component() } };
        let _: View<G> = view! { ComponentWithChildren { div {} } };
        let _: View<G> = view! { ComponentWithChildren { div {} div {} } };
        let _: View<G> = view! { ComponentWithChildren { Component {} } };
        let _: View<G> = view! { ComponentWithChildren() { Component {} } };
        let _: View<G> = view! { ComponentWithChildren {} };
        let _: View<G> = view! { ComponentWithChildren() };
        let _: View<G> = view! { ComponentWithChildren() {} };
        let _: View<G> = view! { AttributesComponent(attr:class = "test") {} };
        let str_signal = create_signal(String::new());
        let _: View<G> = view! { AttributesComponent(bind:value = str_signal) {} };
        let on_click = |_| {};
        let _: View<G> = view! { AttributesComponent(on:click = on_click) {} };
        let bool_signal = create_signal(false);
        let _: View<G> =
            view! { AttributesComponent(attr:disabled = false, attr:checked = bool_signal.get()) };

        let _: View<G> = view! {
            AsyncComponentWithPropsDestructuring {
                Component {}
            }
        };

        let _: View<G> = view! {
            NestedComponentWithChildren {
                Component {}
            }
        };
    });
}

fn main() {}
