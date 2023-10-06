use gloo_timers::future::TimeoutFuture;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;

#[component]
fn TimerCounter<G: Html>() -> View<G> {
    let mut state = create_signal(0);

    spawn_local_scoped(async move {
        loop {
            TimeoutFuture::new(1000).await;
            state += 1;
        }
    });

    view! {
        div {
            p { "Value: " (state.get()) }
        }
    }
}

fn main() {
    sycamore::render(TimerCounter);
}
