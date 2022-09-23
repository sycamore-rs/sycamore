use log::Level;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use wasm_bindgen::prelude::*;
mod error;
use error::ResultToJsResult;

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
    let result = create_rc_signal(String::new());

    match fetch_visits(id).await {
        Ok(visit) => result.set(visit.value.to_string()),
        Err(e) => {
            if let Some(err) = e.as_string() {
                result.set(err);
            } else {
                result.set("Network error".into());
            }
        }
    };

    view! { cx,
        p {
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
