use sycamore::prelude::*;

#[component(App<G>)]
fn app() -> TemplateResult<G> {
    let name = Signal::new(String::new());

    let handle_change = move |_| unreachable!();

    template! {
        div {
            h1 {
                "Hello "
                ({if !name.get().is_empty() {
                    cloned!((name) => template! {
                        span { (name.get()) }
                    })
                } else {
                    template! { span { "World" } }
                }})
                "!"
            }

            input(placeholder="What is your name?", on:input=handle_change)
        }
    }
}

fn main() {
    let s = render_to_string(|| template! { App() });
    println!("{}", s);
}
