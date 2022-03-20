use gloo_timers::future::TimeoutFuture;
use sycamore::futures::ScopeSpawnLocal;
use sycamore::prelude::*;

#[component]
fn TimerCounter<G: Html>(cx: Scope) -> View<G> {
    let state = cx.create_signal(0);

    cx.spawn_local(async move {
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
