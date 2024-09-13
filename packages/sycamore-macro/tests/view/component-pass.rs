use sycamore::prelude::*;

#[component]
pub fn Component() -> View {
    view! {
        div {}
    }
}

#[derive(Props)]
pub struct Props {
    prop: &'static str,
}

#[component]
pub fn PropsComponent(Props { prop: _ }: Props) -> View {
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
pub fn AllDefaultPropsComponent(_props: AllDefaultProps) -> View {
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
pub fn OptionalPropsComponent(_props: OptionalProps) -> View {
    view! {
        div {}
    }
}

#[derive(Props)]
pub struct PropsWithChildren {
    children: Children,
}

#[component]
pub fn ComponentWithChildren(props: PropsWithChildren) -> View {
    let children = props.children.call();

    view! {
        div {
            (children)
        }
    }
}

#[component]
pub fn NestedComponentWithChildren(props: PropsWithChildren) -> View {
    let children = props.children.call();

    view! {
        ComponentWithChildren {
            (children)
            Component {}
        }
    }
}

#[component]
pub async fn AsyncComponentWithPropsDestructuring(
    PropsWithChildren { children }: PropsWithChildren,
) -> View {
    children.call()
}

#[derive(Props)]
pub struct AttributesProps {
    #[prop(attributes(html, input))]
    attributes: Attributes,
}

#[component]
pub fn AttributesComponent(AttributesProps { attributes }: AttributesProps) -> View {
    view! {
        input(..attributes) {}
    }
}

fn compile_pass() {
    let _ = create_root(|| {
        let _: View = view! { Component() };
        let _: View = view! { Component {} };

        let prop = "prop";
        let _: View = view! { PropsComponent(prop=prop) };

        let _: View = view! { AllDefaultPropsComponent(prop=123) };
        let _: View = view! { AllDefaultPropsComponent() };
        let _: View = view! { AllDefaultPropsComponent {} };

        let _: View = view! { OptionalPropsComponent(optional=123) };
        let _: View = view! { OptionalPropsComponent(implicit=123) };
        let _: View = view! { OptionalPropsComponent(optional=123, implicit=123) };
        let _: View = view! { OptionalPropsComponent() };
        let _: View = view! { OptionalPropsComponent {} };

        let _: View = view! { ComponentWithChildren { Component() } };
        let _: View = view! { ComponentWithChildren { div {} } };
        let _: View = view! { ComponentWithChildren { div {} div {} } };
        let _: View = view! { ComponentWithChildren { Component {} } };
        let _: View = view! { ComponentWithChildren() { Component {} } };
        let _: View = view! { ComponentWithChildren {} };
        let _: View = view! { ComponentWithChildren() };
        let _: View = view! { ComponentWithChildren() {} };

        let _: View = view! { AttributesComponent(class="test") {} };
        let str_signal = create_signal(String::new());
        let _: View = view! { AttributesComponent(bind:value=str_signal) {} };
        let on_click = |_| {};
        let _: View = view! { AttributesComponent(on:click=on_click) {} };
        let bool_signal = create_signal(false);
        let _: View = view! { AttributesComponent(disabled=false, checked=bool_signal.get()) };
        // input specific attribute.
        let _: View = view! { AttributesComponent(value="text") };

        let _: View = view! {
            AsyncComponentWithPropsDestructuring {
                Component {}
            }
        };

        let _: View = view! {
            NestedComponentWithChildren {
                Component {}
            }
        };
    });
}

fn main() {}
