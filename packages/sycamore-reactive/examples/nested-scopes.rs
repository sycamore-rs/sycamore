use sycamore_reactive::*;

fn main() {
    create_scope_immediate(|ctx| {
        let outer = ctx.create_signal(0);
        let disposer = ctx.create_child_scope(|ctx| {
            dbg!(outer.get());
            ctx.create_effect(|| {
                dbg!(outer.get());
            });
        });
        outer.set(1);
        disposer();
        // Doesn't call the effect because it has been disposed.
        outer.set(2);
    });
}
