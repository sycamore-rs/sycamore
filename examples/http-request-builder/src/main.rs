use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use sycamore::prelude::*;
use sycamore::suspense::{Suspense, SuspenseProps};
use sycamore::web::tags::*;

// API that counts visits to the web-page
const API_BASE_URL: &str = "https://abacus.jasoncameron.dev/hit";

#[derive(Serialize, Deserialize, Default, Debug)]
struct Visits {
    value: u64,
}

async fn fetch_visits(id: &str) -> Result<Visits, reqwasm::Error> {
    let url = format!("{API_BASE_URL}/{id}/http-request-builder");
    let resp = Request::get(&url).send().await?;

    resp.json::<Visits>().await
}

#[component]
async fn VisitsCount() -> View {
    let id = "sycamore-builder-visits-counter";
    let visits = fetch_visits(id).await.unwrap_or_default();

    p().children(("Total Visits: ", span().children(move || visits.value)))
        .into()
}

#[component]
fn App() -> View {
    div()
        .children((
            p().children("Page Visit Counter"),
            Suspense(
                SuspenseProps::builder()
                    .fallback("Loading".into())
                    .children(Children::new(VisitsCount))
                    .build(),
            ),
        ))
        .into()
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(App);
}
