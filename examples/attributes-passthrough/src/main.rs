use sycamore::prelude::*;

#[derive(Props)]
pub struct AccessibleInputProps {
    label_id: &'static str,
    // attributes: Attributes<G>,
    children: Children,
}

#[component]
fn AccessibleSearchBox(props: AccessibleInputProps) -> View {
    let _ = props.label_id;
    let _ = props.children;
    // props
    //     .attributes
    //     .exclude_keys(&["aria-role", "aria-labelledby"]);
    // let children = props.children.call();
    //
    // view! {
    //     input(aria-role="searchbox", aria-labelledby=props.label_id, ..props.attributes) {
    //         (children)
    //     }
    // }
    todo!("attributes passthrough")
}

#[component]
fn App() -> View {
    view! {
        div {
            p { "Passthrough attributes demo" }

            div {
                label(id="searchbox1_label") { "Search Box 1" }
                // AccessibleSearchBox(label_id="searchbox1_label", attr:style="background-color:red;") {}
                AccessibleSearchBox(label_id="searchbox1_label") {}
            }

            div {
                label(id="searchbox2_label") { "Search Box 2" }
                // AccessibleSearchBox(label_id="searchbox2_label", attr:style="background-color:yellow;") { }
            }
        }
    }
}

fn main() {
    sycamore::render(App);
}
