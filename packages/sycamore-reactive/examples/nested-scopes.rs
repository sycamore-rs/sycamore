use sycamore_reactive::*;

fn main() {
    create_scope_immediate(|cx| {
        let outer = create_signal(cx, 0);
        let disposer = create_child_scope(cx, |cx| {
            dbg!(outer.get());
            create_effect(cx, || {
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
