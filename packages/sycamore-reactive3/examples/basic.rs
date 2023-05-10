use sycamore_reactive3::*;

fn main() {
    let root = create_root(|cx| {
        let signal = create_signal(cx, 1);
        let doubled = create_memo(cx, move || signal.get() * 2);
        let doubled_as_well = create_memo(cx, move || signal() + signal());
        let quadrupled = create_memo(cx, move || doubled() + doubled_as_well());

        create_effect(cx, move || {
            println!(
                "n = {}, n * 2 = {}, n * 2 (again) = {}, n * 4 = {}",
                signal(),
                doubled(),
                doubled_as_well(),
                quadrupled()
            );
        });

        signal(2);
        doubled.inner_signal().set(10);
        signal(3);
        cx.dispose();
    });
    root.dispose();
}
