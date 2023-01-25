use sycamore::prelude::*;

#[derive(Props)]
pub struct AccessibleInputProps<'a> {
    label_id: &'static str,
    attributes: Attributes<'a>,
    children: Children<'a, G>,
}

#[component]
fn AccessibleSearchBox<'a>(cx: Scope<'a>, props: AccessibleInputProps<'a>) -> View {
    props
        .attributes
        .exclude_keys(&["aria-role", "aria-labelledby"]);
    let children = props.children.call(cx);

    view! { cx,
        input(aria:"role"="searchbox", aria:"labelledby"=props.label_id, ..props.attributes) {
            (children)
        }
    }
}

#[component]
fn App(cx: Scope) -> View {
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
