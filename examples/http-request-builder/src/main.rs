use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use sycamore::builder::prelude::*;
use sycamore::component::Prop;
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

    Ok(resp.json::<Visits>().await?)
}

#[component]
async fn VisitsCount<G: Html>(cx: Scope<'_>) -> View<G> {
    let id = "sycamore-builder-visits-counter";
    let visits = fetch_visits(id).await.unwrap_or_default();

    h(p).t("Total Visits: ")
        .c(h(span).dyn_t(move || visits.value.to_string()))
        .view(cx)
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    h(div)
        .c(h(p).t("Page Visit Counter"))
        .c(Suspense(
            cx,
            // Take advantage that structs that derive Prop have public builders even
            // if the fields are private and come from a different crate (in this case the Sycamore
            // crate).
            SuspenseProps::builder()
                .fallback(t("Loading"))
                .children(Children::new(cx, |cx| VisitsCount(cx, ())))
                .build(),
        ))
        .view(cx)
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|cx| App(cx, ()));
}
