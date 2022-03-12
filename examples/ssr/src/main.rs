use sycamore::prelude::*;

#[component]
fn App<G: Html>(ctx: Scope) -> View<G> {
    let name = ctx.create_signal(String::new());

    let handle_change = move |_| unreachable!();

    view! { ctx,
        div {
            h1 {
                "Hello "
                ({if !name.get().is_empty() {
                    view! { ctx, span { (name.get()) } }
                } else {
                    view! { ctx, span { "World" } }
                }})
                "!"
            }

            input(placeholder="What is your name?", on:input=handle_change)
        }
    }
}

fn main() {
    let s = sycamore::render_to_string(|ctx| view! { ctx, App() });
    println!("{}", s);
}
