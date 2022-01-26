use sycamore_reactive::*;

fn main() {
    create_scope_immediate(|ctx| {
        let data = ctx.create_signal(0);
        let doubled = ctx.create_memo(|| *data.get() * 2);
        ctx.create_effect(|| {
            println!(
                "data value changed. new value = {}, doubled value = {}",
                data.get(),
                doubled.get_untracked() /* FIXME: using get will cause effect to be called twice
                                         * when data is updated */
            )
        });
        data.set(1);
        data.set(2);
        data.set(3);
        data.set(4);
    });
}
