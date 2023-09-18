use sycamore::prelude::*;

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

#[component]
fn Component<G: Html>() -> View<G> {
    view! {
        div {}
    }
}

#[derive(Props)]
pub struct AttributesProps<G: Html> {
    attributes: Attributes<G>,
}

#[component]
pub fn AttributesComponent<G: Html>(
    AttributesProps { attributes: _ }: AttributesProps<G>,
) -> View<G> {
    view! {
        div {}
    }
}

fn compile_fail<G: Html>() {
    let _ = create_root(|| {
        let _: View<G> = view! { UnknownComponent() };
        let _: View<G> = view! { UnknownComponent {} };

        let _: View<G> = view! { Component };
        let _: View<G> = view! { Component(prop=1) };

        let _: View<G> = view! { PropsComponent() };
        let _: View<G> = view! { PropsComponent {} };
        let _: View<G> = view! { PropsComponent(prop=123) };
        let _: View<G> = view! { PropsComponent { prop: "123" } }; // Legacy syntax.
        let _: View<G> = view! { AttributesComponent(attr:class=123) }; // Wrong type
    });
}

fn main() {}
