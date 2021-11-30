use sycamore::prelude::*;

use anyhow::Result;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};

use sycamore::futures::spawn_local_in_scope;

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

#[component(RenderVisits<G>)]
fn render_visits(counter: ReadSignal<u64>) -> View<G> {
    view! {
            div {
                h1 { "Page Visit Counter" }
                p {
                    "Total visits: "
                    (counter.get())
            }
        }
    }
}

#[component(App<G>)]
fn app() -> View<G> {
    let counter = Signal::<u64>::new(0);

    create_effect(cloned!((counter) => move || {
        spawn_local_in_scope(cloned!((counter) => async move {
            let website_id = "page-visit-counter.tyz";
            let visits = fetch_visits(website_id).await.unwrap_or_default();

            counter.set(visits.value);
        }))
    }));

    view! { RenderVisits(counter.handle()) }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|| view! { App() });
}
