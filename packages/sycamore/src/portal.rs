//! Portal API.

use std::any::{Any, TypeId};

use wasm_bindgen::prelude::*;

use crate::prelude::*;

/// Props for [`Portal`].
pub struct PortalProps<G>
where
    G: GenericNode,
{
    pub children: Template<G>,
    pub selector: &'static str,
}

/// A portal into another part of the DOM.
#[component(Portal<G>)]
pub fn portal(props: PortalProps<G>) -> Template<G> {
    let PortalProps { children, selector } = props;

    if TypeId::of::<G>() == TypeId::of::<DomNode>() {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let container = document
            .query_selector(selector)
            .unwrap_throw()
            .expect_throw("could not find element matching selector");

        let children = children.flatten();

        for child in &children {
            container
                .append_child(
                    &<dyn Any>::downcast_ref::<DomNode>(child)
                        .unwrap_throw()
                        .inner_element(),
                )
                .unwrap_throw();
        }

        on_cleanup(move || {
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

    template! {}
}
