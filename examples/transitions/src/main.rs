use gloo_timers::future::TimeoutFuture;
use rand::Rng;
use sycamore::prelude::*;
use sycamore::web::{Suspense, Transition};

#[derive(Debug, Clone, Copy)]
enum Tab {
    One,
    Two,
    Three,
}

impl Tab {
    fn content(self) -> &'static str {
        match self {
            Tab::One => "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Bibendum est ultricies integer quis auctor elit sed. Sed egestas egestas fringilla phasellus faucibus scelerisque eleifend.",
            Tab::Two => "Auctor urna nunc id cursus. Sed viverra tellus in hac habitasse. Non blandit massa enim nec dui nunc mattis. Orci ac auctor augue mauris augue neque. Facilisi cras fermentum odio eu feugiat pretium nibh ipsum.",
            Tab::Three => "Tristique senectus et netus et malesuada fames. Purus in massa tempor nec feugiat nisl pretium fusce. Phasellus faucibus scelerisque eleifend donec. Eget nullam non nisi est sit. Sit amet justo donec enim diam vulputate ut pharetra. Ante in nibh mauris cursus. Quis risus sed vulputate odio ut enim blandit volutpat maecenas.",
        }
    }
}

#[component(inline_props)]
async fn Child(tab: Tab) -> View {
    let delay_ms = rand::thread_rng().gen_range(500..1000);
    TimeoutFuture::new(delay_ms).await;

    view! {
        div {
            p { "Content loaded after " (delay_ms) "ms" }
            p { (tab.content()) }
        }
    }
}

#[component]
fn App() -> View {
    let tab = create_signal(Tab::One);

    let suspense_is_loading = create_signal(true);
    let transition_is_loading = create_signal(true);

    let update = move |x| tab.set(x);

    view! {
        div {
            div {
                button(on:click=move |_| update(Tab::One)) { "One" }
                button(on:click=move |_| update(Tab::Two)) { "Two" }
                button(on:click=move |_| update(Tab::Three)) { "Three" }
            }

            div(style="display: flex; flex-direction: row; gap: 1rem;") {
                div(style="flex: 1 1 0%") {
                    p { strong { "Suspense" } }
                    p { "Suspense state: " (if suspense_is_loading.get() { "loading" } else { "done" }) }
                    Suspense(fallback=|| view! { p { "Loading..." } }, set_is_loading=suspense_is_loading) {
                        ({
                            let tab = tab.get();
                            view! { Child(tab=tab) }
                        })
                    }
                }

                div(style="flex: 1 1 0%") {
                    p { strong { "Transition" } }
                    p { "Transition state: " (if transition_is_loading.get() { "loading" } else { "done" }) }
                    Transition(fallback=|| view! { p { "Loading..." } }, set_is_loading=transition_is_loading) {
                        ({
                            let tab = tab.get();
                            view! { Child(tab=tab) }
                        })
                    }
                }
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    sycamore::render(App);
}
