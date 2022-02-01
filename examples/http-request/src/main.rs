use anyhow::Result;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use sycamore::futures::ScopeFuturesExt;
use sycamore::prelude::*;

// API that counts visits to the web-page
const API_BASE_URL: &str = "https://api.countapi.xyz/hit";

#[derive(Serialize, Deserialize, Default, Debug)]
struct Visits {
    value: u64,
}

async fn fetch_visits(id: &str) -> Result<Visits> {
    let url = format!("{}/{}/hits", API_BASE_URL, id);
    let resp = Request::get(&url).send().await?;

    let body = resp.json::<Visits>().await?;
    Ok(body)
}

#[component]
fn RenderVisits<G: Html>(ctx: ScopeRef, count: u64) -> View<G> {
    view! { ctx,
        div {
            p { "Page Visit Counter" }
            p {
                "Total visits: "
                span(class="text-green-500") {
                    (count)
                }
            }
        }
    }
}

#[component]
fn App<G: Html>(ctx: ScopeRef) -> View<G> {
    let count = ctx.create_resource(async move {
        let website_id = "page-visit-counter-tailwindcss.tyz";
        let visits = fetch_visits(website_id).await.unwrap_or_default();
        visits.value
    });

    view! { ctx, (if let Some(count) = *count.get() {
        view! { ctx, RenderVisits(count) }
    } else {
        view! { ctx, }
    })}
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|ctx| view! { ctx, App {} });
}
