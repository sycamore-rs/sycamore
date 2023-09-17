use sycamore::prelude::*;

#[derive(Props)]
pub struct AccessibleInputProps<G: Html> {
    label_id: &'static str,
    attributes: Attributes<G>,
    children: Children<G>,
}

#[component]
fn AccessibleSearchBox<G: Html>(props: AccessibleInputProps<G>) -> View<G> {
    props
        .attributes
        .exclude_keys(&["aria-role", "aria-labelledby"]);
    let children = props.children.call();

    view! {
        input(aria-role="searchbox", aria-labelledby=props.label_id, ..props.attributes) {
            (children)
        }
    }
}

#[component]
fn App<G: Html>() -> View<G> {
    view! {
        div {
            p { "Passthrough attributes demo" }

            div {
                label(id="searchbox1_label") { "Search Box 1" }
                AccessibleSearchBox(label_id="searchbox1_label", attr:style="background-color:red;") {}
            }

            div {
                label(id="searchbox2_label") { "Search Box 2" }
                AccessibleSearchBox(label_id="searchbox2_label", attr:style="background-color:yellow;") { }
            }
        }
    }
}

fn main() {
    sycamore::render(App);
}
