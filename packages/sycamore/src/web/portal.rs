//! Portal API.

use std::any::Any;

use wasm_bindgen::prelude::*;

use crate::component::Children;
use crate::prelude::*;

/// Props for [`Portal`].
#[derive(Prop, Debug)]
pub struct PortalProps<'a, G>
where
    G: GenericNode,
{
    children: Children<'a, G>,
    selector: &'a str,
}

/// A portal into another part of the DOM.
#[component]
pub fn Portal<'a, G: Html>(cx: Scope<'a>, props: PortalProps<'a, G>) -> View<G> {
    let PortalProps { children, selector } = props;

    if G::IS_BROWSER {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let container = document
            .query_selector(selector)
            .unwrap_throw()
            .expect_throw("could not find element matching selector");

        let children = children.call(cx).flatten();

        for child in &children {
            container
                .append_child(
                    &<dyn Any>::downcast_ref::<DomNode>(child)
                        .unwrap_throw()
                        .inner_element(),
                )
                .unwrap_throw();
        }

        on_cleanup(cx, move || {
            for child in &children {
                container
                    .remove_child(
                        &<dyn Any>::downcast_ref::<DomNode>(child)
                            .unwrap_throw()
                            .inner_element(),
                    )
                    .unwrap_throw();
            }
        });
    } else {
        // TODO: Support for other types of nodes.
    }

    view! { cx, }
}
