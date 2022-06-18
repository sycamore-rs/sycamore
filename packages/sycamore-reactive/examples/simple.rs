use sycamore_reactive::*;

fn main() {
    create_scope_immediate(|cx| {
        let data = create_signal(cx, 0);
        let doubled = create_memo(cx, || *data.get() * 2);
        create_effect(
            cx,
            on([doubled], move || {
                println!("data value changed. new value = {data}, doubled value = {doubled}")
            }),
        );
        data.set(1);
        data.set(2);
        data.set(3);
        data.set(4);
    });
}
