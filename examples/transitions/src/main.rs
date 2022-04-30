use gloo_timers::future::TimeoutFuture;
use rand::Rng;
use sycamore::prelude::*;
use sycamore::suspense::{use_transition, Suspense};

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

#[component]
async fn Child<G: Html>(cx: Scope<'_>, tab: Tab) -> View<G> {
    let delay_ms = rand::thread_rng().gen_range(200..500);
    TimeoutFuture::new(delay_ms).await;

    view! { cx,
        div {
            p { "Content loaded after " (delay_ms) "ms" }
            p { (tab.content()) }
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let tab = create_signal(cx, Tab::One);
    let transition = use_transition(cx);
    let update = move |x| transition.start(move || tab.set(x), || ());

    view! { cx,
        div {
            p { "Suspense + Transitions" }
            p { "Transition state: " (transition.is_pending().then(|| "pending").unwrap_or("done")) }
            button(on:click=move |_| update(Tab::One)) { "One" }
            button(on:click=move |_| update(Tab::Two)) { "Two" }
            button(on:click=move |_| update(Tab::Three)) { "Three" }
            Suspense {
                fallback: view! { cx, p { "Loading..." } },
                ({
                    let tab = *tab.get();
                    view! { cx, Child(tab) }
                })
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|cx| view! { cx, App {} });
}
