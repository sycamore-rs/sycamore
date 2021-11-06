use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> View<G> {
    let name = Signal::new(String::new());

    let handle_change = move |_| unreachable!();

    view! {
        div {
            h1 {
                "Hello "
                ({if !name.get().is_empty() {
                    cloned!((name) => view! {
                        span { (name.get()) }
                    })
                } else {
                    view! { span { "World" } }
                }})
                "!"
            }

            input(placeholder="What is your name?", on:input=handle_change)
        }
    }
}

fn main() {
    let s = sycamore::render_to_string(|| view! { App() });
    println!("{}", s);
}
