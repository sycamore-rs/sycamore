use sycamore_reactive::*;

fn main() {
    create_scope_immediate(|cx| {
        let data = cx.create_signal(123);
        dbg!(data.get());
        let signal_ref = cx.create_signal(data);
        dbg!(signal_ref.get().get());
    });
}
