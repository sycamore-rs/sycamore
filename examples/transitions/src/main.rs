use gloo_timers::future::TimeoutFuture;
use rand::Rng;
use sycamore::prelude::*;
use sycamore::web::{create_client_resource, Suspense, Transition};

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

async fn get_content(tab: Tab) -> &'static str {
    let delay_ms = rand::thread_rng().gen_range(500..1000);
    TimeoutFuture::new(delay_ms).await;

    tab.content()
}

#[component(inline_props)]
fn TabContent(content: &'static str) -> View {
    view! {
        p { (content) }
    }
}

#[component]
fn App() -> View {
    let tab = create_signal(Tab::One);
    let content = create_client_resource(on(tab, move || get_content(tab.get())));

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
                    Suspense(fallback=|| view! { p { "Loading..." } }, set_is_loading=move |is_loading| suspense_is_loading.set(is_loading)) {
                        (if let Some(content) = content.value.get() {
                            view! { TabContent(content=content) }
                        } else {
                            view! {}
                        })
                    }
                }

                div(style="flex: 1 1 0%") {
                    p { strong { "Transition" } }
                    p { "Transition state: " (if transition_is_loading.get() { "loading" } else { "done" }) }
                    Transition(fallback=|| view! { p { "Loading..." } }, set_is_loading=move |is_loading| transition_is_loading.set(is_loading)) {
                        (if let Some(content) = content.value.get() {
                            view! { TabContent(content=content) }
                        } else {
                            view! {}
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
