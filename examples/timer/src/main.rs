use gloo_timers::future::TimeoutFuture;
use sycamore::futures::ScopeSpawnLocal;
use sycamore::prelude::*;

#[component]
fn TimerCounter<G: Html>(ctx: Scope) -> View<G> {
    let state = ctx.create_signal(0);

    ctx.spawn_local(async move {
        loop {
            TimeoutFuture::new(1000).await;
            state.set(*state.get() + 1);
        }
    });

    view! { ctx,
        div {
            p { "Value: " (state.get()) }
        }
    }
}

fn main() {
    sycamore::render(|ctx| {
        view! { ctx, TimerCounter {} }
    });
}
