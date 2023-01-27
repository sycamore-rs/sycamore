use sycamore::builder::input;
use sycamore::prelude::*;

#[derive(Props)]
pub struct AccessibleInputProps<'a> {
    label_id: &'static str,
    attributes: Attributes<'a, input>,
    children: Children<'a, WebNode>,
}

#[component]
fn AccessibleSearchBox<'a>(cx: Scope<'a>, props: AccessibleInputProps<'a>) -> View {
    let children = props.children.call(cx);

    view! { cx,
        input("aria-role"="searchbox", "aria-labelledby"=props.label_id, ..props.attributes) {
            (children)
        }
    }
}

#[component]
fn App(cx: Scope) -> View {
    view! { cx,
        div {
            p { "Passthrough attributes demo" }

            div {
                label(id = "searchbox1_label") { "Search Box 1" }
                AccessibleSearchBox(label_id = "searchbox1_label", attr:style="background-color:red;") {}
            }
            div {
                label(id = "searchbox2_label") { "Search Box 2" }
                AccessibleSearchBox(label_id = "searchbox2_label", attr:style="background-color:yellow;") { }
            }
        }
    }
}

fn main() {
    sycamore::render(App);
}
