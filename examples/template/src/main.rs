use gloo_timers::future::TimeoutFuture;
use sycamore::futures::spawn_local_scoped;
use sycamore::generic_node::{Template, TemplateId, TemplateShape};
use sycamore::prelude::*;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let mut state = create_signal(cx, 0i64);

    spawn_local_scoped(cx, async move {
        loop {
            TimeoutFuture::new(1000).await;
            state.set(*state.get() + 1);
        }
    });

    let template = Template {
        id: TemplateId(0),
        shape: TemplateShape::Element {
            ident: "div",
            ns: None,
            children: &[
                TemplateShape::Text("Hello World!"),
                TemplateShape::DynMarker,
                TemplateShape::Text("Goodbye!"),
                TemplateShape::Element {
                    ident: "button",
                    ns: None,
                    children: &[TemplateShape::Text("Click me.")],
                    attributes: &[],
                    flag: true,
                },
            ],
            attributes: &[],
            flag: false,
        },
    };
    let dyn_values = vec![View::new_dyn(cx, move || view! { cx, p { (state.get()) } })];
    let result = G::instantiate_template(template);
    G::apply_dyn_values_to_template(cx, &result.dyn_markers, dyn_values);
    
    result.flagged_nodes[0].event(cx, "click", move |_| {
        state *= 2;
    });

    View::new_node(result.root)
}

fn main() {
    sycamore::render(App);
}
