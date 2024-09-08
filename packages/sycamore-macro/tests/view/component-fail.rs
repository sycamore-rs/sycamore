use sycamore::prelude::*;

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

#[component]
fn Component() -> View {
    view! {
        div {}
    }
}

#[derive(Props)]
pub struct AttributesProps {
    attributes: Attributes,
}

#[component]
pub fn AttributesComponent(AttributesProps { attributes: _ }: AttributesProps<G>) -> View {
    view! {
        div {}
    }
}

fn compile_fail() {
    let _ = create_root(|| {
        let _: View = view! { UnknownComponent() };
        let _: View = view! { UnknownComponent {} };

        let _: View = view! { Component };
        let _: View = view! { Component(prop=1) };

        let _: View = view! { PropsComponent() };
        let _: View = view! { PropsComponent {} };
        let _: View = view! { PropsComponent(prop=123) };
        let _: View = view! { PropsComponent { prop: "123" } }; // Legacy syntax.
        let _: View = view! { AttributesComponent(attr:class=123) }; // Wrong type
    });
}

fn main() {}
