use gloo_timers::future::TimeoutFuture;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;

#[component]
fn App(cx: Scope) -> View {
    let state = create_signal(cx, 0);

    spawn_local_scoped(cx, async move {
        loop {
            TimeoutFuture::new(1000).await;
            state.set(*state.get() + 1);
        }
    });

    view! { cx,
        div {
            p { "Value: " (state.get()) }
        }
    }
}

fn main() {
    sycamore::render(App);
}
