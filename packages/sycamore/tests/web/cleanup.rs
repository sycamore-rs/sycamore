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
    let root = create_scope(|ctx| {
        ctx.on_cleanup(on_cleanup_callback);
    });
    assert_cleanup_called(|| {
        root();
    });
}

#[wasm_bindgen_test]
pub fn test_cleanup_in_effect() {
    create_scope_immediate(|ctx| {
        let trigger = ctx.create_signal(());
        ctx.create_effect_scoped(|ctx| {
            trigger.track();
            ctx.on_cleanup(on_cleanup_callback);
        });

        assert_cleanup_called(|| {
            trigger.set(());
        });
    });
}

#[component]
fn CleanupComp<G: Html>(ctx: ScopeRef) -> View<G> {
    ctx.on_cleanup(on_cleanup_callback);
    view! { ctx, }
}

#[wasm_bindgen_test]
fn component_cleanup_on_root_destroyed() {
    let root = create_scope(|ctx| {
        let _: View<DomNode> = view! { ctx,
            CleanupComp {}
        };
    });

    assert_cleanup_called(move || {
        root();
    });
}
