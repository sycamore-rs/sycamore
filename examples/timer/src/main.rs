use gloo_timers::future::TimeoutFuture;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;

#[component]
fn TimerCounter<G: Html>(cx: Scope) -> View<G> {
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
    sycamore::render(|cx| {
        view! { cx, TimerCounter {} }
    });
}
