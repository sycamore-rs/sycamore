use gloo_timers::future::TimeoutFuture;
use sycamore::futures::spawn_local_scoped;
use sycamore::generic_node::{Template, TemplateId, TemplateShape};
use sycamore::prelude::*;
use sycamore::utils::render::insert;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let state = create_signal(cx, 0);

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
                    flag: true,
                },
            ],
            flag: false,
        },
    };
    let result = G::instantiate_template(template);

    let dynamic_values = vec![View::new_dyn(cx, move || view! { cx, p { (state.get()) } })];
    for (m, value) in result.dyn_markers.iter().zip(dynamic_values.into_iter()) {
        insert(cx, &m.parent, value, None, m.before.as_ref(), m.multi);
    }
    result.flagged_nodes[0].event(cx, "click", |_| {
        state.set(*state.get() * 2);
    });
    View::new_node(result.root)
}

fn main() {
    sycamore::render(App);
}
