use log::{info, Level};
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;

// API that counts visits to the web-page
const API_BASE_URL: &str = "https://api.countapi.xyz/hit";

#[derive(Serialize, Deserialize, Default, Debug)]
struct Visits {
    value: u64,
}

async fn fetch_visits(id: &str) -> Result<Visits, reqwasm::Error> {
    let url = format!("{}/{}/hits", API_BASE_URL, id);
    let resp = Request::get(&url).send().await?;
    let body = resp.json::<Visits>().await?;
    Ok(body)
}

#[component]
async fn VisitsCount<G: Html>(cx: Scope<'_>) -> View<G> {
    let error_message = create_signal(cx, String::new());
    // Fetching default value is `true` since we are fetching on window load as well.
    let fetching = create_signal(cx, true);
    let fetch_is_error = create_signal(cx, false);
    let id = "sycamore-visits-counter";
    let visits = create_signal(cx, Visits::default());

    let fetch = move || {
        spawn_local_scoped(cx, async {
            info!("Start fetching...");
            match fetch_visits(id).await {
                Ok(value) => visits.set(value), // Set new visits value
                Err(value) => {
                    // Notify that an error occured here.
                    fetch_is_error.set(true);
                    // Set error message
                    error_message.set(value.to_string());
                }
            }
            // Notify that we are done fetching.
            info!("Done.");
            fetching.set(false);
        });
    };

    // Fetch on window load
    fetch();

    // Retry fetching here
    let start_fetching = move |_| {
        // Make sure to check fetching is completed before retrying.
        if !*fetching.get() {
            // Notify that we are about to start fetching again.
            fetching.set(true);
            fetch();
        }
    };

    view! { cx,
        p {
            button (on:click=start_fetching) {
                (if *fetching.get() {
                    "fetching..."
                } else {
                    "retry?"
                })
            }
            br {}
            br {}
            span {
                (if *fetch_is_error.get() {
                    error_message.get().as_ref().into()
                } else if *fetching.get() {
                    "".into()
                } else {
                    format!("Total visits: {}", visits.get().value)
                })
            }
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div {
            p { "Page Visit Counter" }
            Suspense(fallback=view! { cx, "Loading..." }) {
                VisitsCount {}
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).unwrap();
    sycamore::render(|cx| view! { cx, App {} });
}
