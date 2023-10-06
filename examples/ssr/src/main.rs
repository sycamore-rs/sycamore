use sycamore::prelude::*;

#[component]
fn App<G: Html>() -> View<G> {
    let name = create_signal(String::new());

    let handle_change = move |_| unreachable!();

    view! {
        div {
            h1 {
                "Hello "
                ({if !name.with(String::is_empty) {
                    view! { span { (name.get_clone()) } }
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
    let s = sycamore::render_to_string(App);
    println!("{}", s);
}
