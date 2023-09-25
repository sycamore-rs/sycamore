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
    let root = create_root(|| {
        on_cleanup(on_cleanup_callback);
    });
    assert_cleanup_called(|| root.dispose());
}

#[wasm_bindgen_test]
pub fn test_cleanup_in_effect() {
    let _ = create_root(|| {
        let trigger = create_signal(());
        create_effect(move || {
            trigger.track();
            on_cleanup(on_cleanup_callback);
        });

        assert_cleanup_called(|| {
            trigger.set(());
        });
    });
}

#[component]
fn CleanupComp<G: Html>() -> View<G> {
    on_cleanup(on_cleanup_callback);
    view! {}
}

#[wasm_bindgen_test]
fn component_cleanup_on_root_destroyed() {
    let root = create_root(|| {
        let _: View<DomNode> = view! {
            CleanupComp {}
        };
    });

    assert_cleanup_called(move || root.dispose());
}
