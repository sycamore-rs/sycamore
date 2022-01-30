use std::time::Duration;

use sycamore::easing;
use sycamore::motion::ScopeMotionExt;
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

#[component]
fn Tweened<G: Html>(ctx: ScopeRef, _: ()) -> View<G> {
    let progress =
        ctx.create_tweened_signal([0.0f32, 1.0], Duration::from_millis(250), easing::quad_out);

    view! { ctx,
        div {
            style {
                r#"
                progress {
                    display: block;
                    width: 100%;
                }
                "#
            }
            progress(value=progress.get()[0])
            progress(value=progress.get()[1])

            button(on:click=|_| progress.set([0.0, 1.0])) { "0%" }
            button(on:click=|_| progress.set([0.25, 0.75])) { "25%" }
            button(on:click=|_| progress.set([0.5, 0.5])) { "50%" }
            button(on:click=|_| progress.set([0.75, 0.25])) { "75%" }
            button(on:click=|_| progress.set([1.0, 0.0])) { "100%" }
        }
    }
}

fn main() {
    sycamore::render(|ctx| {
        view! { ctx,
            p { "Motion demo" }
            p { "request_animation_frame" }
            CreateRAF {}
            br
            p { "Tweened signals" }
            Tweened {}
        }
    });
}
