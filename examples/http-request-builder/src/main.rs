use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use sycamore::builder::*;
use sycamore::prelude::*;
use sycamore::suspense::{Suspense, SuspenseProps};

// API that counts visits to the web-page
const API_BASE_URL: &str = "https://api.countapi.xyz/hit";

#[derive(Serialize, Deserialize, Default, Debug)]
struct Visits {
    value: u64,
}

async fn fetch_visits(id: &str) -> Result<Visits, reqwasm::Error> {
    let url = format!("{}/{}/hits", API_BASE_URL, id);
    let resp = Request::get(&url).send().await?;

    resp.json::<Visits>().await
}

#[component]
async fn VisitsCount(cx: Scope<'_>) -> View {
    let id = "sycamore-builder-visits-counter";
    let visits = fetch_visits(id).await.unwrap_or_default();

    p(cx)
        .child("Total Visits: ")
        .child(span(cx).dyn_child(move |_| visits.value.to_string()))
        .view()
}

#[component]
fn App(cx: Scope) -> View {
    div(cx)
        .child(p(cx).child("Page Visit Counter"))
        .child(Suspense(
            cx,
            SuspenseProps::builder()
                .fallback("Loading".to_view(cx))
                .children(Children::new(|cx| VisitsCount(cx)))
                .build(),
        ))
        .view()
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(App);
}
