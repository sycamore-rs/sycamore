use sycamore::prelude::*;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let name = cx.create_signal(String::new());

    let handle_change = move |_| unreachable!();

    view! { cx,
        div {
            h1 {
                "Hello "
                ({if !name.get().is_empty() {
                    view! { cx, span { (name.get()) } }
                } else {
                    view! { cx, span { "World" } }
                }})
                "!"
            }

            input(placeholder="What is your name?", on:input=handle_change)
        }
    }
}

fn main() {
    let s = sycamore::render_to_string(|cx| view! { cx, App() });
    println!("{}", s);
}
