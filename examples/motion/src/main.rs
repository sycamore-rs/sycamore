use std::time::Duration;

use sycamore::easing;
use sycamore::motion::{create_raf, create_tweened_signal};
use sycamore::prelude::*;

#[component]
fn CreateRAF(cx: Scope) -> View {
    let state = create_signal(cx, 0i32);
    let (_running, start, stop) = create_raf(cx, || {
        state.set(*state.get() + 1);
    });
    view! { cx,
        div {
            p { (state.get()) " frames" }
            button(on:click=move |_| start()) { "Start" }
            button(on:click=move |_| stop()) { "Stop" }
        }
    }
}

#[component]
fn Tweened(cx: Scope) -> View {
    let progress = create_tweened_signal(
        cx,
        [0.0f32, 1.0],
        Duration::from_millis(250),
        easing::quad_out,
    );

    view! { cx,
        div {
            style {
                r#"
                progress {
                    display: block;
                    width: 100%;
                }
                "#
            }
            progress(prop:"value"=progress.get()[0])
            progress(prop:"value"=progress.get()[1])

            button(on:click=|_| progress.set([0.0, 1.0])) { "0%" }
            button(on:click=|_| progress.set([0.25, 0.75])) { "25%" }
            button(on:click=|_| progress.set([0.5, 0.5])) { "50%" }
            button(on:click=|_| progress.set([0.75, 0.25])) { "75%" }
            button(on:click=|_| progress.set([1.0, 0.0])) { "100%" }
        }
    }
}

fn main() {
    sycamore::render(|cx| {
        view! { cx,
            p { "Motion demo" }
            strong { "request_animation_frame" }
            CreateRAF {}
            br {}
            strong { "Tweened signals" }
            Tweened {}
        }
    });
}
