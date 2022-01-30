use sycamore::prelude::*;

#[component]
fn App<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    view! { ctx,
        p {
            "Hello World!"
        }
    }
}

fn main() {
    sycamore::render(|ctx| {
        view! { ctx,
            App()
        }
    });
}
