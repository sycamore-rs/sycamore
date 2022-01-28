use sycamore::portal::Portal;

use super::*;

#[wasm_bindgen_test]
fn test_portal() {
    let test_container = test_container();

    let portal_target = document().create_element("div").unwrap();
    portal_target.set_id("portal-target");
    test_container.append_child(&portal_target).unwrap();

    let root = document().create_element("div").unwrap();
    test_container.append_child(&root).unwrap();

    create_scope_immediate(|ctx| {
        let switch = ctx.create_signal(true);
        sycamore::render_to(
            |_| {
                view! { ctx,
                    (if *switch.get() {
                        view! { ctx,
                            Portal {
                                selector: "#portal-target",
                                "Hello from the other side!"
                            }
                        }
                    } else {
                        view! { ctx, }
                    })
                }
            },
            &root,
        );
        assert_eq!(portal_target.inner_html(), "Hello from the other side!");

        // Destroying the portal should remove the portal from the DOM.
        switch.set(false);
        // TODO: if/else does not actually create a new scope.
        // assert_eq!(portal_target.inner_html(), "");
    });
}
