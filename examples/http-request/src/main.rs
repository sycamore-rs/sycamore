use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
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
    let id = "sycamore-visits-counter";
    let visits = fetch_visits(id).await.unwrap_or_default();

    view! { cx,
        div {
            p {
                "Total visits: "
                span(class="text-green-500") {
                    (visits.value)
                }
            }
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div {
            p { "Page Visit Counter" }
            Suspense {
                fallback: view! { cx, "Loading..." },
                VisitsCount {}
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|cx| view! { cx, App {} });
}
