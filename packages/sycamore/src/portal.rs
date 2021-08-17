//! Portal API.

use std::any::{Any, TypeId};

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
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container = document
            .query_selector(selector)
            .unwrap()
            .expect("could not find element matching selector");

        let children = children.flatten();

        for child in &children {
            container
                .append_child(
                    &<dyn Any>::downcast_ref::<DomNode>(child)
                        .unwrap()
                        .inner_element(),
                )
                .unwrap();
        }

        on_cleanup(move || {
            for child in &children {
                container
                    .remove_child(
                        &<dyn Any>::downcast_ref::<DomNode>(child)
                            .unwrap()
                            .inner_element(),
                    )
                    .unwrap();
            }
        });
    } else {
        // TODO: Support for other types of nodes.
    }

    template! {}
}
