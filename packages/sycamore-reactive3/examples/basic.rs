use sycamore_reactive3::signals::create_signal;
use sycamore_reactive3::*;

fn main() {
    let root = create_root(|cx| {
        let signal = create_signal(cx, 123);
        println!("{}", signal());
        signal.set(456);
        println!("{}", signal());
        cx.dispose();
        println!("{}", signal());
    });
}
