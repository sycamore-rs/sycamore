use sycamore::generic_node::{Template, TemplateId, TemplateShape};
use sycamore::prelude::*;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let mut state = create_signal(cx, 0i64);

    static TEMPLATE: Template = Template {
        id: TemplateId(0),
        shape: TemplateShape::Element {
            tag: "div",
            ns: None,
            children: &[
                TemplateShape::Element {
                    tag: "p",
                    ns: None,
                    children: &[TemplateShape::Text("Value: "), TemplateShape::DynMarker],
                    attributes: &[],
                    flag: false,
                },
                TemplateShape::Element {
                    tag: "button",
                    ns: None,
                    children: &[TemplateShape::Text("+")],
                    attributes: &[],
                    flag: true,
                },
                TemplateShape::Element {
                    tag: "button",
                    ns: None,
                    children: &[TemplateShape::Text("-")],
                    attributes: &[],
                    flag: true,
                },
                TemplateShape::Element {
                    tag: "button",
                    ns: None,
                    children: &[TemplateShape::Text("Reset")],
                    attributes: &[],
                    flag: true,
                },
            ],
            attributes: &[],
            flag: false,
        },
    };
    let dyn_values = vec![view! { cx, (state.get()) }];
    let result = G::instantiate_template(&TEMPLATE);
    G::apply_dyn_values_to_template(cx, &result.dyn_markers, dyn_values);

    result.flagged_nodes[0].event(cx, "click", move |_| state += 1);
    result.flagged_nodes[1].event(cx, "click", move |_| state -= 1);
    result.flagged_nodes[2].event(cx, "click", move |_| state.set(0));

    View::new_node(result.root)
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let string = sycamore::render_to_string(App);
    log::info!("{string}");

    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap()
        .set_inner_html(&string);

    sycamore::hydrate(App);
}
