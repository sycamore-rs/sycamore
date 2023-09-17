use std::time::Duration;

use sycamore::easing;
use sycamore::motion::{create_raf, create_tweened_signal};
use sycamore::prelude::*;

#[component]
fn CreateRAF<G: Html>() -> View<G> {
    let mut state = create_signal(0i32);
    let (_running, start, stop) = create_raf(move || {
        state += 1;
    });
    view! {
        div {
            p { (state.get()) " frames" }
            button(on:click=move |_| start()) { "Start" }
            button(on:click=move |_| stop()) { "Stop" }
        }
    }
}

#[component]
fn Tweened<G: Html>() -> View<G> {
    let progress =
        create_tweened_signal([0.0f32, 1.0], Duration::from_millis(250), easing::quad_out);

    view! {
        div {
            style {
                r#"
                progress {
                    display: block;
                    width: 100%;
                }
                "#
            }
            progress(prop:value=progress.get()[0])
            progress(prop:value=progress.get()[1])

            button(on:click=move |_| progress.set([0.0, 1.0])) { "0%" }
            button(on:click=move |_| progress.set([0.25, 0.75])) { "25%" }
            button(on:click=move |_| progress.set([0.5, 0.5])) { "50%" }
            button(on:click=move |_| progress.set([0.75, 0.25])) { "75%" }
            button(on:click=move |_| progress.set([1.0, 0.0])) { "100%" }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();

    sycamore::render(|| {
        view! {
            p { "Motion demo" }
            p { "request_animation_frame" }
            CreateRAF {}
            br {}
            p { "Tweened signals" }
            Tweened {}
        }
    });
}
