use sycamore::prelude::*;

#[derive(Props)]
pub struct AccessibleInputProps<'cx, G: Html> {
    label_id: &'static str,
    #[prop(default)]
    attributes: Attributes<'cx, G>,
    children: Children<'cx, G>,
}

#[component]
fn AccessibleSearchBox<'cx, G: Html>(
    cx: Scope<'cx>,
    mut props: AccessibleInputProps<'cx, G>,
) -> View<G> {
    props
        .attributes
        .exclude_keys(&["aria-role", "aria-labelledby"]);
    let children = props.children.call(cx);

    view! { cx,
        input(aria-role = "searchbox", aria-labelledby = props.label_id, ..props.attributes) {
            (children)
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div {
            "Passthrough attributes demo"

            label(id = "searchbox1_label") { "Search Box 1" }
            AccessibleSearchBox(label_id = "searchbox1_label", attr:style="background-color:slategray;") {}
            label(id = "searchbox2_label") { "Search Box 2" }
            AccessibleSearchBox(label_id = "searchbox2_label", attr:style="background-color:gray;") { }
        }
    }
}

fn main() {
    sycamore::render(|cx| view! { cx, App {} });
}
