use std::cell::RefCell;
use std::rc::Rc;

use sycamore_reactive::*;

fn main() {
    let trigger = create_rc_signal(());
    let trigger_cloned = trigger.clone();
    let disposer = Rc::new(RefCell::new(None::<Box<dyn FnOnce()>>));
    let tmp = Rc::clone(&disposer);

    let f = create_scope(move |ctx| {
        let data = ctx.create_signal(0);
        ctx.create_effect(move || {
            trigger.track();
            if let Some(tmp) = tmp.take() {
                tmp();
            }
            let a = data.get();
            dbg!(a);
        });
    });
    *disposer.borrow_mut() = Some(Box::new(f));
    trigger_cloned.set(());
}
