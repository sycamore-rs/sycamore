use sycamore_reactive::*;

fn main() {
    create_scope_immediate(|ctx| {
        let data = ctx.create_signal(123);
        dbg!(data.get());
        let signal_ref = ctx.create_signal(data);
        dbg!(signal_ref.get().get());
    });
}
