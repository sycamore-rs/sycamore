use sycamore_web2::*;

fn app() -> View {
    let state = create_signal(0);
    div()
        .class("p-5")
        .children((
            button()
                .on(events::click, move |_ev| state.update(|x| *x += 1))
                .class("bg-red-300 p-2 rounded")
                .children("Increment"),
            div()
                .id("counter-div")
                .class(move || format!("counter-{state} font-bold"))
                .children((move || format!("Counter: {state}"), "!")),
            move || {
                if state.get() % 2 == 0 {
                    p().children("Even")
                } else {
                    p().children("Odd")
                }
            },
            div().class("flex flex-right").children((
                div().children(indexed_test()),
                div().class("ml-5").children(keyed_test()),
            )),
        ))
        .into()
}

fn indexed_test() -> View {
    let list = create_signal(vec!["a", "b", "c"]);
    let add = move |_ev| list.update(|x| x.push("test".into()));
    let clear = move |_ev| list.set(vec![]);
    let remove_one = move |_ev| {
        list.update(|x| x.pop());
    };

    (
        p().children("Indexed list:"),
        button()
            .class("border-2 border-black p-2 rounded")
            .on(events::click, add)
            .children("Add"),
        button()
            .class("ml-1 border-2 border-black p-2 rounded")
            .on(events::click, clear)
            .children("Clear"),
        button()
            .class("ml-1 border-2 border-black p-2 rounded")
            .on(events::click, remove_one)
            .disabled(move || list.with(|x| x.is_empty()))
            .children("Pop"),
        // ul().children(Indexed(list, |item| li().children(item))),
        ul().children(Indexed(IndexedProps::builder().list(list).view(|item| li().children(item)).build(),
    )
        .into()
}

fn keyed_test() -> View {
    let mut next = create_signal(0);
    let list = create_signal(vec!["First".to_string()]);
    let add = move |_ev| {
        list.update(|x| x.push(format!("test-{}", next.get())));
        next += 1;
    };
    let clear = move |_ev| list.set(vec![]);
    let swap = move |_ev| {
        if list.with(|x| x.len()) > 1 {
            list.update(|x| {
                let len = x.len();
                x.swap(0, len - 1);
            });
        }
    };
    let remove_one = move |_ev| {
        list.update(|x| x.pop());
    };
    let cycle = move |_ev| {
        list.update(|x| {
            let len = x.len();
            if len > 1 {
                let last = x.pop().unwrap();
                x.insert(0, last);
            }
        });
    };

    (
        p().children("Keyed list:"),
        button()
            .class("border-2 border-black p-2 rounded")
            .on(events::click, add)
            .children("Add"),
        button()
            .class("ml-1 border-2 border-black p-2 rounded")
            .on(events::click, clear)
            .children("Clear"),
        button()
            .class("ml-1 border-2 border-black p-2 rounded")
            .on(events::click, swap)
            .children("Swap"),
        button()
            .class("ml-1 border-2 border-black p-2 rounded")
            .disabled(move || list.with(|x| x.is_empty()))
            .on(events::click, remove_one)
            .children("Pop"),
        button()
            .class("ml-1 border-2 border-black p-2 rounded")
            .on(events::click, cycle)
            .children("Cycle"),
        ul().children(Keyed(list, |item| li().children(item), |item| item.clone())),
    )
        .into()
}

fn main() {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let html = render_to_string(app);
    web_sys::console::log_1(&html.clone().into());

    let dom_root = document.query_selector("#container").unwrap().unwrap();
    render_to(app, &dom_root);

    let hydrate_root = document.query_selector("#hydrate-test").unwrap().unwrap();
    hydrate_root.set_inner_html(&html);
    hydrate_to(app, &hydrate_root);
}
