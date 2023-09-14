use sycamore_reactive3::*;

fn main() {
    let root = create_root(|| {
        let signal = create_signal(1);
        let doubled = create_memo(move || signal.get() * 2);
        let doubled_as_well = create_memo(move || signal() + signal());
        let quadrupled = create_memo(move || doubled() + doubled_as_well());

        create_effect(move || {
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
    });
    root.dispose();
}
