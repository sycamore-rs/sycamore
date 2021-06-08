use std::time::Duration;

use sycamore::rx::Tweened;
use sycamore::{easing, prelude::*};

#[component(App<G>)]
fn app() -> Template<G> {
    let progress = Tweened::new([0.0, 1.0], Duration::from_millis(250), easing::quad_out);
    let progress0 = progress.clone();
    let progress1 = progress.clone();
    let progress2 = progress.clone();
    let progress3 = progress.clone();
    let progress4 = progress.clone();
    let progress5 = progress.clone();

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
            progress(value=progress.get()[0])
            progress(value=progress0.get()[1])

            button(on:click=move |_| progress1.set([0.0, 1.0])) { "0%" }
            button(on:click=move |_| progress2.set([0.25, 0.75])) { "25%" }
            button(on:click=move |_| progress3.set([0.5, 0.5])) { "50%" }
            button(on:click=move |_| progress4.set([0.75, 0.25])) { "75%" }
            button(on:click=move |_| progress5.set([1.0, 0.0])) { "100%" }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
