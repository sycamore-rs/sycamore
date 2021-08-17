use sycamore::portal::{Portal, PortalProps};

use super::*;

#[wasm_bindgen_test]
fn test_portal() {
    let test_container = test_container();

    let el = document().create_element("div").unwrap();
    el.set_id("portal-target");
    test_container.append_child(&el).unwrap();

    let el2 = document().create_element("div").unwrap();
    test_container.append_child(&el2).unwrap();

    let portal = Signal::new(None);

    sycamore::render_to(
        cloned!((portal) => move || {
            portal.set(Some(template! {
                Portal(PortalProps {
                    children: template! { "Hello World!" },
                    selector: "#portal-target",
                })
            }));
            template! {
                (portal.get().as_ref().as_ref().cloned().unwrap_or_default())
            }
        }),
        &el2,
    );

    assert_eq!(el.inner_html(), "Hello World!");

    // Destroying the portal should remove the portal from the DOM.
    // portal.set(None);

    // assert_eq!(el.inner_html(), "");
}
