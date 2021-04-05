#![allow(non_snake_case)]

use std::time::Duration;

use maple_core::reactive::Tweened;
use maple_core::{easing, prelude::*};

fn App<G: GenericNode>() -> TemplateResult<G> {
    let progress = Tweened::new(0.0, Duration::from_millis(250), easing::cubic_out);
    let progress1 = progress.clone();
    let progress2 = progress.clone();
    let progress3 = progress.clone();
    let progress4 = progress.clone();

    template! {
        div {
            style {
                r#"
                progress {
                    display: block;
                    width: 100%;
                }
                "#
            }
            progress(value=progress.get())

            button(on:click=move |_| progress1.set(0.0)) { "0%" }
            button(on:click=move |_| progress2.set(0.25)) { "25%" }
            button(on:click=move |_| progress3.set(0.5)) { "50%" }
            button(on:click=move |_| progress4.set(1.0)) { "100%" }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
