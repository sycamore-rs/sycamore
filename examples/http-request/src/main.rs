use log::{info, Level};
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use sycamore::futures::spawn_local;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use wasm_bindgen::prelude::*;
mod utils;
use utils::{SplitCloned, ToJsResult};

// API that counts visits to the web-page
const API_BASE_URL: &str = "https://api.countapi.xyz/hit";

#[derive(Serialize, Deserialize, Default, Debug)]
struct Visits {
    value: u64,
}

async fn fetch_visits(id: &str) -> Result<Visits, JsValue> {
    let url = format!("{}/{}/hits", API_BASE_URL, id);
    let resp = Request::get(&url).send().await.into_js_result()?;
    let body = resp.json::<Visits>().await.into_js_result()?;
    Ok(body)
}

#[component]
async fn VisitsCount<G: Html>(cx: Scope<'_>) -> View<G> {
    let id = "sycamore-visits-counter";
    // For more ergonomic using split_cloned;
    let (fetching, cloned_fetching) = create_rc_signal(true).split_cloned();
    let (result, cloned_result) = create_rc_signal(String::new()).split_cloned();
    let (result_is_ok, cloned_result_is_ok) = create_rc_signal(true).split_cloned();

    let fetch = move || {
        // Reclone all the necessary stuff before spawning local async move.
        let moved_fetching = cloned_fetching.clone();
        let moved_result = cloned_result.clone();
        let moved_result_is_ok = cloned_result_is_ok.clone();
        spawn_local(async move {
            info!("Start fetching...");
            match fetch_visits(id).await {
                Ok(visit) => moved_result.set(visit.value.to_string()), // Set result value
                Err(e) => {
                    // Notify that an error occured here.
                    moved_result_is_ok.set(false);

                    // Set error message
                    if let Some(err) = e.as_string() {
                        moved_result.set(err);
                    } else {
                        moved_result.set("Unexpected Network error!".into());
                    }
                }
            }
            // Notify that we are done fetching.
            info!("Done.");
            moved_fetching.set(false);
        });
    };

    // Fetch now at initialization
    fetch();

    let (fetching, moved_fetching) = fetching.split_cloned();

    // Retry fetching here
    let start_fetching = move |_| {
        // Make sure to check fetching is completed before retrying.
        if !*moved_fetching.get() {
            // Notify that we are about to start fetching again.
            moved_fetching.set(true);
            fetch();
        }
    };

    let (fetching, cloned_fetching) = fetching.split_cloned();

    view! { cx,
        p {
            button (on:click=start_fetching) {
                (if *cloned_fetching.get() {
                    "fetching..."
                } else {
                    "retry?"
                })
            }
            br {}
            br {}
            span {
                (if !*fetching.get() {
                    if *result_is_ok.get() {
                        "Request Ok"
                    } else {
                        "Request Err"
                    }
                } else {
                    ""
                })
            }
            br {}
            br {}
            "Total visits: "
            span {
                (*result.get())
            }
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let loading = view! { cx, "Loading..." };
    view! { cx,
        div {
            p { "Page Visit Counter" }
            Suspense(fallback=loading) {
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
