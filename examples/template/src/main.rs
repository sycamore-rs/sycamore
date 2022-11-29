use gloo_timers::future::TimeoutFuture;
use sycamore::futures::spawn_local_scoped;
use sycamore::generic_node::{instantiate_template_universal, Template, TemplateId, TemplateShape};
use sycamore::prelude::*;
use sycamore::utils::render::insert;

#[component]
fn App(cx: Scope) -> View<DomNode> {
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
            ],
            flag: false,
        },
    };
    let result = instantiate_template_universal(template);
    let dynamic = View::new_dyn(cx, move || view! { cx, (state.get()) });
    let _0 = &result.dyn_markers[0];
    insert(cx, &_0.parent, dynamic, None, _0.before.as_ref(), _0.multi);
    View::new_node(result.root)
}

fn main() {
    sycamore::render(App);
}
