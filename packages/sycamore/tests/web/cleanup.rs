use std::cell::Cell;

use super::*;

thread_local!(static CLEANUP_CALLED: Cell<bool> = Cell::new(false));
fn assert_cleanup_called(f: impl FnOnce()) {
    CLEANUP_CALLED.with(|cleanup_called| {
        assert!(!cleanup_called.get());
        f();
        assert!(cleanup_called.get());

        cleanup_called.set(false); // Reset for next test
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

    assert_cleanup_called(|| {
        drop(root);
    });
}

#[wasm_bindgen_test]
pub fn test_cleanup_in_effect() {
    let trigger = Signal::new(());

    create_effect(cloned!((trigger) => move || {
        trigger.get();
        on_cleanup(on_cleanup_callback);
    }));

    assert_cleanup_called(|| {
        trigger.set(());
    });
}

#[component(CleanupComp<G>)]
fn comp() -> Template<G> {
    on_cleanup(on_cleanup_callback);

    template! {}
}

#[wasm_bindgen_test]
fn component_cleanup_on_root_destroyed() {
    let root = create_root(|| {
        let _: Template<DomNode> = template! {
            CleanupComp()
        };
    });

    assert_cleanup_called(move || {
        drop(root);
    });
}
