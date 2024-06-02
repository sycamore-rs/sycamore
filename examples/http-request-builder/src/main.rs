use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use sycamore::builder::prelude::*;
use sycamore::component::Props;
use sycamore::prelude::*;
use sycamore::suspense::{Suspense, SuspenseProps};

// API that counts visits to the web-page
const API_BASE_URL: &str = "https://abacus.jasoncameron.dev/hit";

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
async fn VisitsCount<G: Html>() -> View<G> {
    let id = "sycamore-builder-visits-counter";
    let visits = fetch_visits(id).await.unwrap_or_default();

    p().t("Total Visits: ")
        .c(span().dyn_t(move || visits.value.to_string()))
        .view()
}

#[component]
fn App<G: Html>() -> View<G> {
    div()
        .c(p().t("Page Visit Counter"))
        .c(Suspense(
            SuspenseProps::builder()
                .fallback(t("Loading"))
                .children(Children::new(|| VisitsCount()))
                .build(),
        ))
        .view()
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(App);
}
