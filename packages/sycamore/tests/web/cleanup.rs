use std::cell::Cell;

use super::*;

thread_local!(static CLEANUP_CALLED: Cell<bool> = Cell::new(false));
fn assert_cleanup_called(f: impl FnOnce()) {
    CLEANUP_CALLED.with(|cleanup_called| {
        cleanup_called.set(false);
        f();
        assert!(cleanup_called.get());
    });
}

fn on_cleanup_callback() {
    CLEANUP_CALLED.with(|cleanup_called| {
        cleanup_called.set(true);
    });
}

#[wasm_bindgen_test]
pub fn test_cleanup_in_root() {
    let root = create_scope(|cx| {
        on_cleanup(cx, on_cleanup_callback);
    });
    assert_cleanup_called(|| unsafe {
        root.dispose();
    });
}

#[wasm_bindgen_test]
pub fn test_cleanup_in_effect() {
    create_scope_immediate(|cx| {
        let trigger = create_signal(cx, ());
        create_effect_scoped(cx, |cx| {
            trigger.track();
            on_cleanup(cx, on_cleanup_callback);
        });

        assert_cleanup_called(|| {
            trigger.set(());
        });
    });
}

#[component]
fn CleanupComp<G: Html>(cx: Scope) -> View<G> {
    on_cleanup(cx, on_cleanup_callback);
    view! { cx, }
}

#[wasm_bindgen_test]
fn component_cleanup_on_root_destroyed() {
    let root = create_scope(|cx| {
        let _: View<DomNode> = view! { cx,
            CleanupComp {}
        };
    });

    assert_cleanup_called(move || unsafe {
        root.dispose();
    });
}
