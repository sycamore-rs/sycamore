use sycamore_reactive3::{*, signals::create_signal};

fn main() {
    let root = create_root(|cx| {
        let signal = create_signal(cx, 123);
        println!("{}", signal.get());
        signal.set(456);
        println!("{}", signal.get());
    });
}
