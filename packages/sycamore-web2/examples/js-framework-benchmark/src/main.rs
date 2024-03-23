#![allow(non_snake_case)]

use std::sync::atomic::{AtomicUsize, Ordering};

use rand::prelude::*;
use sycamore_web2::*;

static ADJECTIVES: &[&str] = &[
    "pretty",
    "large",
    "big",
    "small",
    "tall",
    "short",
    "long",
    "handsome",
    "plain",
    "quaint",
    "clean",
    "elegant",
    "easy",
    "angry",
    "crazy",
    "helpful",
    "mushy",
    "odd",
    "unsightly",
    "adorable",
    "important",
    "inexpensive",
    "cheap",
    "expensive",
    "fancy",
];

static COLOURS: &[&str] = &[
    "red", "yellow", "blue", "green", "pink", "brown", "purple", "brown", "white", "black",
    "orange",
];

static NOUNS: &[&str] = &[
    "table", "chair", "house", "bbq", "desk", "car", "pony", "cookie", "sandwich", "burger",
    "pizza", "mouse", "keyboard",
];

fn Button(id: &'static str, text: &'static str, callback: Box<dyn Fn()>) -> View {
    div()
        .class("col-sm-6 smallpad")
        .children(
            button()
                .id(id)
                .class("btn btn-primary btn-block")
                .on(events::click, move |_| callback())
                .children(text),
        )
        .into()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RowData {
    id: usize,
    label: Signal<String>,
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn build_data(count: usize) -> Vec<RowData> {
    let mut thread_rng = thread_rng();

    let mut data = Vec::new();
    data.reserve_exact(count);

    for _i in 0..count {
        let adjective = ADJECTIVES.choose(&mut thread_rng).unwrap();
        let colour = COLOURS.choose(&mut thread_rng).unwrap();
        let noun = NOUNS.choose(&mut thread_rng).unwrap();
        let capacity = adjective.len() + colour.len() + noun.len() + 2;
        let mut label = String::with_capacity(capacity);
        label.push_str(adjective);
        label.push(' ');
        label.push_str(colour);
        label.push(' ');
        label.push_str(noun);

        data.push(RowData {
            id: ID_COUNTER.load(Ordering::Relaxed),
            label: create_signal(label),
        });

        ID_COUNTER.store(ID_COUNTER.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    }

    data
}

fn App() -> View {
    let data = create_signal(Vec::<RowData>::new());
    let selected = create_signal(None::<usize>);

    let remove = move |id| data.update(|d| d.retain(|row| row.id != id));

    let run = move || {
        selected.set(None);
        data.set(build_data(1000));
    };

    let runlots = move || {
        selected.set(None);
        data.set(build_data(10000));
    };

    let add = move || {
        let new = build_data(1000);
        data.update(|d| d.extend(new));
    };

    let update = move || {
        let d = data.get_clone();
        // data.with(|d| {
        for row in d.into_iter().step_by(10) {
            row.label.update(|l| *l = format!("{} !!!", l));
        }
        // })
    };

    let clear = move || {
        data.set(Vec::new());
        selected.set(None);
    };

    let swaprows = move || {
        data.update(|d| {
            if d.len() > 998 {
                d.swap(1, 998);
            }
        })
    };

    div()
        .class("container")
        .children((
            div().class("jumbotron").children(
                div().class("row").children((
                    div()
                        .class("col-md-6")
                        .children(h1().children("Sycamore Keyed")),
                    div()
                        .class("col-md-6")
                        .children(div().class("row").children((
                            Button("run", "Create 1,000 rows", Box::new(run)),
                            Button("runlots", "Create 10,000 rows", Box::new(runlots)),
                            Button("add", "Append 1,000 rows", Box::new(add)),
                            Button("update", "Update every 10th row", Box::new(update)),
                            Button("clear", "Clear", Box::new(clear)),
                            Button("swaprows", "Swap Rows", Box::new(swaprows)),
                        ))),
                )),
            ),
            table()
                .class("table table-hover table-striped test-data")
                .children(tbody().children(Keyed(
                    data,
                    move |row| {
                        let is_selected = create_selector(move || selected.get() == Some(row.id));
                        let handle_click = move |_| selected.set(Some(row.id));
                        on_cleanup(move || {
                            row.label.dispose();
                        });
                        tr().class(move || if is_selected.get() { "danger" } else { "" })
                            .children((
                                td().class("col-md-1").children(row.id.to_string()),
                                td().class("col-md-4").children(
                                    a().on(events::click, handle_click)
                                        .children(move || row.label.get_clone()),
                                ),
                                td().class("col-md-1").children(
                                    a().on(events::click, move |_| remove(row.id)).children(
                                        span()
                                            .class("glyphicon glyphicon-remove")
                                            .attr("aria-hidden", "true"),
                                    ),
                                ),
                                td().class("col-md-6"),
                            ))
                    },
                    |row| row.id,
                ))),
        ))
        .into()
}

fn main() {
    let document = web_sys::window().unwrap().document().unwrap();
    let mount_el = document.query_selector("#main").unwrap().unwrap();

    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    render_to(App, &mount_el);
}
