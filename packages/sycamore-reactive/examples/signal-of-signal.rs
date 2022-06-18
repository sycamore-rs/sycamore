use sycamore_reactive::*;

fn main() {
    create_scope_immediate(|cx| {
        let data = create_signal(cx, 123);
        dbg!(data.get());
        let signal_ref = create_signal(cx, data);
        dbg!(signal_ref.get().get());
    });
}
