//! Portal API.

use std::any::Any;

use wasm_bindgen::prelude::*;

use crate::prelude::*;

/// Props for [`Portal`].
#[derive(Props, Debug)]
pub struct PortalProps<G>
where
    G: GenericNode,
{
    children: Children<G>,
    #[prop(setter(into))]
    selector: String,
}

/// A portal into another part of the DOM.
#[component]
pub fn Portal<G: Html>(props: PortalProps<G>) -> View<G> {
    let PortalProps { children, selector } = props;

    if G::IS_BROWSER {
        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let container = document
            .query_selector(&selector)
            .unwrap_throw()
            .expect_throw("could not find element matching selector");

        let children = children.call().flatten();

        for child in &children {
            container
                .append_child(
                    &<dyn Any>::downcast_ref::<DomNode>(child)
                        .unwrap_throw()
                        .to_web_sys(),
                )
                .unwrap_throw();
        }

        on_cleanup(move || {
            for child in &children {
                container
                    .remove_child(
                        &<dyn Any>::downcast_ref::<DomNode>(child)
                            .unwrap_throw()
                            .to_web_sys(),
                    )
                    .unwrap_throw();
            }
        });
    } else {
        // TODO: Support for other types of nodes.
    }

    view! {}
}
