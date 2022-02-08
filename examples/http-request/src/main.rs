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
async fn VisitsCount<G: Html>(ctx: ScopeRef<'_>) -> View<G> {
    let id = "sycamore-visits-counter";
    let visits = fetch_visits(id).await.unwrap_or_default();

    view! { ctx,
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
fn App<G: Html>(ctx: ScopeRef) -> View<G> {
    view! { ctx,
        div {
            p { "Page Visit Counter" }
            Suspense {
                fallback: view! { ctx, "Loading..." },
                VisitsCount {}
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|ctx| view! { ctx, App {} });
}
