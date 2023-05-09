use sycamore_reactive3::memos::create_memo;
use sycamore_reactive3::signals::create_signal;
use sycamore_reactive3::*;

fn main() {
    let root = create_root(|cx| {
        let signal = create_signal(cx, 1);
        let doubled = create_memo(cx, move || signal.get() * 2);
        let doubled_as_well = create_memo(cx, move || signal.get() + signal.get());
        let quadrupled = create_memo(cx, move || doubled.get() + doubled_as_well.get());
        let _print = create_memo(cx, move || {
            println!(
                "n = {}, n * 2 = {}, n * 2 (again) = {}, n * 4 = {}",
                signal.get(),
                doubled.get(),
                doubled_as_well.get(),
                quadrupled.get()
            );
        });
        signal.set(123);
        signal.set(456);
    });
    root.dispose();
}
