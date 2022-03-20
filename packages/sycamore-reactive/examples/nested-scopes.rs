use sycamore_reactive::*;

fn main() {
    create_scope_immediate(|cx| {
        let outer = cx.create_signal(0);
        let disposer = cx.create_child_scope(|cx| {
            dbg!(outer.get());
            cx.create_effect(|| {
                dbg!(outer.get());
            });
        });
        outer.set(1);
        unsafe {
            disposer.dispose();
        }
        // Doesn't call the effect because it has been disposed.
        outer.set(2);
    });
}
