use sycamore::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cat {
    id: &'static str,
    name: &'static str,
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let items = create_signal(
        cx,
        vec![
            Cat {
                id: "J---aiyznGQ",
                name: "Keyboard Cat",
            },
            Cat {
                id: "z_AbfPXTKms",
                name: "Maru",
            },
            Cat {
                id: "OUtn3pvWmpg",
                name: "Henri The Existential Cat",
            },
        ],
    );

    view! { cx,
        p { "The famous cats of YouTube" }
        ul {
            Indexed {
                iterable: items,
                view: |cx, Cat { id, name }| view! { cx,
                    li {
                        a(href=format!("https://www.youtube.com/watch?v={id}")) {
                            (name)
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    sycamore::render(|cx| view! { cx, App() });
}
