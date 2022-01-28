use sycamore::motion::ScopeCreateRaf;
use sycamore::prelude::*;

#[component]
fn CreateRAF<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let state = ctx.create_signal(0i32);
    let (_running, start, stop) = ctx.create_raf(|| {
        state.set(*state.get() + 1);
    });
    view! { ctx,
        div {
            p { (state.get()) " frames" }
            button(on:click=|_| start()) { "Start" }
            button(on:click=|_| stop()) { "Stop" }
        }
    }
}

fn main() {
    sycamore::render(|ctx| {
        view! { ctx,
            "Motion demo"
            CreateRAF {}
        }
    });
}
